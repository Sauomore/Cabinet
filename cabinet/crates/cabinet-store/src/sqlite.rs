//! SQLite 后端实现
//!
//! 使用 SQLite 作为结构化文件容器：
//! - 元数据表：docs, meta
//! - WAL 表：wal（顺序追加）
//! - 索引表：index_drawer_XX（16 张表，B-tree 主键即 (sim, abs) 合并的 u16）

use crate::{Backend, StoreConfig, StoreError, WalRecord, WalType};
use cabinet_hsh::HSHCode;
use cabinet_index::posting::PostingList;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct SQLiteBackend {
    conn: Connection,
    config: StoreConfig,
}

impl Backend for SQLiteBackend {
    fn open(path: &Path, config: &StoreConfig) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let backend = SQLiteBackend {
            conn,
            config: config.clone(),
        };
        backend.init_tables()?;
        Ok(backend)
    }

    fn append_wal(&self, record: &WalRecord) -> Result<(), StoreError> {
        let hsh_bytes = HSHCode::encode_seq(&record.hsh_seq);
        let checksum = crc32fast::hash(&hsh_bytes);
        self.conn.execute(
            "INSERT INTO wal (record_type, timestamp_ms, doc_id, hsh_seq_bytes, checksum)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.record_type as u8,
                record.timestamp_ms as i64,
                record.doc_id as i64,
                hsh_bytes,
                checksum as i64
            ],
        )?;
        if self.config.wal_sync {
            // SQLite 的 PRAGMA synchronous = FULL 已在 init 中设置
        }
        Ok(())
    }

    fn read_wal(&self) -> Result<Vec<WalRecord>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT record_type, timestamp_ms, doc_id, hsh_seq_bytes FROM wal ORDER BY seq")?;
        let rows = stmt.query_map([], |row| {
            let rt: u8 = row.get(0)?;
            let ts: i64 = row.get(1)?;
            let doc_id: i64 = row.get(2)?;
            let hsh_bytes: Vec<u8> = row.get(3)?;
            Ok((rt, ts, doc_id, hsh_bytes))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (rt, ts, doc_id, hsh_bytes) = row?;
            let hsh_seq = HSHCode::decode_seq(&hsh_bytes).unwrap_or_default();
            records.push(WalRecord {
                record_type: WalType::from_u8(rt).unwrap_or(WalType::Insert),
                timestamp_ms: ts as u64,
                doc_id: doc_id as u64,
                hsh_seq,
            });
        }
        Ok(records)
    }

    fn write_token(&self, doc_id: u64, hsh_seq: &[HSHCode]) -> Result<(), StoreError> {
        let hsh_bytes = HSHCode::encode_seq(hsh_seq);
        self.conn.execute(
            "INSERT OR REPLACE INTO docs (id, hsh_seq_bytes, created_at)
             VALUES (?1, ?2, ?3)",
            params![
                doc_id as i64,
                hsh_bytes,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64
            ],
        )?;
        Ok(())
    }

    fn write_posting(
        &self,
        feat: u8,
        key: u16,
        postings: &PostingList,
    ) -> Result<(), StoreError> {
        let table = format!("index_drawer_{:02X}", feat);
        let bytes = postings.to_bytes();
        self.conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (sim_abs, postings_bytes) VALUES (?1, ?2)",
                table
            ),
            params![key as i64, bytes],
        )?;
        Ok(())
    }

    fn read_postings(
        &self,
        feat: u8,
        key_range: std::ops::Range<u16>,
    ) -> Result<Vec<(u16, PostingList)>, StoreError> {
        let table = format!("index_drawer_{:02X}", feat);
        let mut stmt = self.conn.prepare(&format!(
            "SELECT sim_abs, postings_bytes FROM {}
             WHERE sim_abs BETWEEN ?1 AND ?2
             ORDER BY sim_abs",
            table
        ))?;
        let rows = stmt.query_map(
            params![key_range.start as i64, key_range.end as i64],
            |row| {
                let key: i64 = row.get(0)?;
                let bytes: Vec<u8> = row.get(1)?;
                Ok((key as u16, bytes))
            },
        )?;

        let mut results = Vec::new();
        for row in rows {
            let (key, bytes) = row?;
            if let Some(pl) = PostingList::from_bytes(&bytes) {
                results.push((key, pl));
            }
        }
        Ok(results)
    }

    fn atomic_replace(&self, _old: &Path, _new: &Path) -> Result<(), StoreError> {
        // SQLite 中通过事务 + RENAME 实现原子替换
        // 简化：暂不实现（LSM 合并可用事务替代）
        Ok(())
    }

    fn snapshot(&self, dst: &Path) -> Result<(), StoreError> {
        self.conn.execute(
            "VACUUM INTO ?1",
            params![dst.to_string_lossy().as_ref()],
        )?;
        Ok(())
    }

    fn read_token(&self, doc_id: u64) -> Result<Option<Vec<HSHCode>>, StoreError> {
        let result: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT hsh_seq_bytes FROM docs WHERE id = ?1",
                params![doc_id as i64],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result.and_then(|b| HSHCode::decode_seq(&b)))
    }
}

impl SQLiteBackend {
    fn init_tables(&self) -> Result<(), StoreError> {
        // 启用 WAL 模式（SQLite 自带 WAL，与我们的 WAL 表不同）
        self.conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             CREATE TABLE IF NOT EXISTS meta (
                 key TEXT PRIMARY KEY,
                 value TEXT
             );
             CREATE TABLE IF NOT EXISTS wal (
                 seq INTEGER PRIMARY KEY AUTOINCREMENT,
                 record_type INTEGER NOT NULL,
                 timestamp_ms INTEGER NOT NULL,
                 doc_id INTEGER NOT NULL,
                 hsh_seq_bytes BLOB NOT NULL,
                 checksum INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS docs (
                 id INTEGER PRIMARY KEY,
                 hsh_seq_bytes BLOB NOT NULL,
                 created_at INTEGER NOT NULL
             );
            "
        )?;

        // 创建 16 个 drawer 表
        for feat in 0..16u8 {
            let table = format!("index_drawer_{:02X}", feat);
            self.conn.execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                        sim_abs INTEGER PRIMARY KEY,
                        postings_bytes BLOB NOT NULL
                    )",
                    table
                ),
                [],
            )?;
        }

        Ok(())
    }
}


mod tests {
    use super::*;
    use cabinet_hsh::HSHCode;
    use std::time::SystemTime;

    #[test]
    fn test_sqlite_backend() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let config = StoreConfig::default();
        let backend = SQLiteBackend::open(&path, &config).unwrap();

        // 写入 WAL
        let record = WalRecord {
            record_type: WalType::Insert,
            timestamp_ms: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            doc_id: 1,
            hsh_seq: vec![HSHCode::new(0, 1, 2)],
        };
        backend.append_wal(&record).unwrap();

        // 读取 WAL
        let records = backend.read_wal().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].doc_id, 1);

        // 写入 Token
        backend.write_token(1, &record.hsh_seq).unwrap();
        let token = backend.read_token(1).unwrap();
        assert!(token.is_some());
        assert_eq!(token.unwrap(), record.hsh_seq);
    }

    #[test]
    fn test_sqlite_posting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let config = StoreConfig::default();
        let backend = SQLiteBackend::open(&path, &config).unwrap();

        let mut pl = PostingList::new();
        pl.add(1, 0);
        pl.add(1, 5);
        pl.add(2, 3);
        backend.write_posting(0x0, 0x4201, &pl).unwrap();

        let results = backend.read_postings(0x0, 0x4200..0x4300).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0x4201);
        assert_eq!(results[0].1.doc_count, 2);
    }
}
