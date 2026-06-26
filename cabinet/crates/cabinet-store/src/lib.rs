//! 存储层：Backend Trait 与 SQLite 实现
//!
//! 抽象存储细节，支持 SQLite（MVP）和 RocksDB（v1.0）。

use cabinet_hsh::HSHCode;
use cabinet_index::posting::PostingList;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("SQLite 错误: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("WAL 损坏: {0}")]
    WalCorrupt(String),
    #[error("配置错误: {0}")]
    Config(String),
}

/// 存储配置
pub struct StoreConfig {
    pub path: String,
    pub backend_type: BackendType,
    pub wal_sync: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            path: "./agent_memory.db".to_string(),
            backend_type: BackendType::SQLite,
            wal_sync: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    SQLite,
    RocksDB,
}

/// WAL 记录
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalRecord {
    pub record_type: WalType,
    pub timestamp_ms: u64,
    pub doc_id: u64,
    pub hsh_seq: Vec<HSHCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalType {
    Insert = 0x01,
    Delete = 0x02,
    Checkpoint = 0x03,
}

impl WalType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(WalType::Insert),
            0x02 => Some(WalType::Delete),
            0x03 => Some(WalType::Checkpoint),
            _ => None,
        }
    }
}

/// 存储后端 Trait
pub trait Backend: Send + Sync {
    /// 初始化/打开存储
    fn open(path: &Path, config: &StoreConfig) -> Result<Self, StoreError>
    where
        Self: Sized;

    /// 写入 WAL 记录（顺序追加，无锁）
    fn append_wal(&self, record: &WalRecord) -> Result<(), StoreError>;

    /// 读取 WAL（崩溃恢复）
    fn read_wal(&self) -> Result<Vec<WalRecord>, StoreError>;

    /// 写入 Token（Layer 1）
    fn write_token(&self, doc_id: u64, hsh_seq: &[HSHCode]) -> Result<(), StoreError>;

    /// 写入/合并 PostingList（Layer 2）
    fn write_posting(
        &self,
        feat: u8,
        key: u16,
        postings: &PostingList,
    ) -> Result<(), StoreError>;

    /// 读取 PostingList（前缀扫描）
    fn read_postings(
        &self,
        feat: u8,
        key_range: std::ops::Range<u16>,
    ) -> Result<Vec<(u16, PostingList)>, StoreError>;

    /// 原子替换（LSM 合并后）
    fn atomic_replace(&self, old: &Path, new: &Path) -> Result<(), StoreError>;

    /// 快照（备份）
    fn snapshot(&self, dst: &Path) -> Result<(), StoreError>;

    /// 读取单个 Token 的 HSH 序列
    fn read_token(&self, doc_id: u64) -> Result<Option<Vec<HSHCode>>, StoreError>;
}

// SQLite 实现放在 sqlite.rs 模块中
pub mod sqlite;

pub use sqlite::SQLiteBackend;

impl Clone for StoreConfig {
    fn clone(&self) -> Self {
        StoreConfig {
            path: self.path.clone(),
            backend_type: self.backend_type,
            wal_sync: self.wal_sync,
        }
    }
}
