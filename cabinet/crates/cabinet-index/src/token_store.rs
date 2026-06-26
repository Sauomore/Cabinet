//! Token Store：Layer 1，原始文档的 HSH 序列，append-only
//!
//! 存储每个文档的原始 HSH 编码序列，用于 decode 和精确匹配。

use cabinet_hsh::HSHCode;

/// 单个 Token 记录
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRecord {
    pub doc_id: u64,
    pub hsh_seq: Vec<HSHCode>,
    pub timestamp: u64,
}

/// Token Store：内存中的追加缓冲区
/// 当缓冲区满时触发 merge 到 ArchiveIndex
pub struct TokenStore {
    buffer: Vec<TokenRecord>,
    threshold: usize,
    next_doc_id: u64,
}

impl TokenStore {
    pub fn new(threshold: usize) -> Self {
        TokenStore {
            buffer: Vec::with_capacity(threshold),
            threshold,
            next_doc_id: 1,
        }
    }

    /// 插入文档，返回分配的 doc_id
    pub fn insert(&mut self, hsh_seq: Vec<HSHCode>) -> u64 {
        let doc_id = self.next_doc_id;
        self.next_doc_id += 1;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.buffer.push(TokenRecord {
            doc_id,
            hsh_seq,
            timestamp,
        });
        doc_id
    }

    /// 是否达到合并阈值
    pub fn should_merge(&self) -> bool {
        self.buffer.len() >= self.threshold
    }

    /// 提取缓冲区用于合并（清空自身）
    pub fn drain(&mut self) -> Vec<TokenRecord> {
        std::mem::take(&mut self.buffer)
    }

    /// 获取文档的 HSH 序列（用于 decode）
    pub fn get(&self, doc_id: u64) -> Option<&[HSHCode]> {
        self.buffer
            .iter()
            .find(|r| r.doc_id == doc_id)
            .map(|r| r.hsh_seq.as_slice())
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cabinet_hsh::HSHCode;

    #[test]
    fn test_token_store() {
        let mut store = TokenStore::new(3);
        let id1 = store.insert(vec![HSHCode::new(0x0, 0x1, 0x2)]);
        let id2 = store.insert(vec![HSHCode::new(0x1, 0x2, 0x3)]);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert!(!store.should_merge());
        store.insert(vec![HSHCode::new(0x2, 0x3, 0x4)]);
        assert!(store.should_merge());

        let drained = store.drain();
        assert_eq!(drained.len(), 3);
        assert!(store.is_empty());
    }
}
