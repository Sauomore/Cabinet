//! PostingList 定义：倒排列表项
//!
//! 每个 (feat, sim, abs) 对应一个 PostingList，记录包含该 HSH 的文档及其位置。

use crate::compress::{decode_delta_u32, encode_delta_u32, decode_vbyte, encode_vbyte};

/// 单个 posting：文档 ID + 文档内位置列表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub doc_id: u64,
    pub positions: Vec<u32>, // 文档内位置（词序号），有序
}

/// PostingList：包含同一 HSH 的所有文档 posting
/// 内部使用 VByte + Delta 压缩存储
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostingList {
    pub doc_count: u32,
    pub postings: Vec<Posting>,
}

impl PostingList {
    pub fn new() -> Self {
        PostingList {
            doc_count: 0,
            postings: Vec::new(),
        }
    }

    pub fn add(&mut self, doc_id: u64, position: u32) {
        if let Some(posting) = self.postings.last_mut() {
            if posting.doc_id == doc_id {
                posting.positions.push(position);
                return;
            }
        }
        self.postings.push(Posting {
            doc_id,
            positions: vec![position],
        });
        self.doc_count += 1;
    }

    /// 合并另一个 PostingList（按 doc_id 归并）
    pub fn merge(&mut self, other: &PostingList) {
        let mut merged = Vec::with_capacity(self.postings.len() + other.postings.len());
        let mut i = 0usize;
        let mut j = 0usize;
        while i < self.postings.len() && j < other.postings.len() {
            let a = &self.postings[i];
            let b = &other.postings[j];
            if a.doc_id < b.doc_id {
                merged.push(a.clone());
                i += 1;
            } else if a.doc_id > b.doc_id {
                merged.push(b.clone());
                j += 1;
            } else {
                // 相同 doc_id，合并位置列表
                let mut positions = a.positions.clone();
                positions.extend_from_slice(&b.positions);
                positions.sort_unstable();
                positions.dedup();
                merged.push(Posting {
                    doc_id: a.doc_id,
                    positions,
                });
                i += 1;
                j += 1;
            }
        }
        merged.extend_from_slice(&self.postings[i..]);
        merged.extend_from_slice(&other.postings[j..]);
        self.postings = merged;
        self.doc_count = self.postings.len() as u32;
    }

    /// 序列化为 VByte + Delta 压缩字节
    /// 格式：[VByte: doc_count] + [postings...]
    /// 每个 posting：[VByte: doc_id] + [VByte: pos_count] + [Delta: positions]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&encode_vbyte(self.doc_count as u64));

        let mut prev_doc_id = 0u64;
        for (i, posting) in self.postings.iter().enumerate() {
            let doc_id_delta = if i == 0 {
                posting.doc_id
            } else {
                posting.doc_id - prev_doc_id
            };
            buf.extend_from_slice(&encode_vbyte(doc_id_delta));
            buf.extend_from_slice(&encode_vbyte(posting.positions.len() as u64));
            buf.extend_from_slice(&encode_delta_u32(&posting.positions));
            prev_doc_id = posting.doc_id;
        }
        buf
    }

    /// 从字节流反序列化
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        let (doc_count, mut offset) = decode_vbyte(bytes)?;
        let doc_count = doc_count as u32;
        let mut postings = Vec::with_capacity(doc_count as usize);
        let mut prev_doc_id = 0u64;

        for _ in 0..doc_count {
            if offset >= bytes.len() {
                return None;
            }
            let (doc_delta, consumed) = decode_vbyte(&bytes[offset..])?;
            offset += consumed;
            let doc_id = prev_doc_id + doc_delta;
            prev_doc_id = doc_id;

            if offset >= bytes.len() {
                return None;
            }
            let (pos_count, consumed) = decode_vbyte(&bytes[offset..])?;
            offset += consumed;
            let pos_count = pos_count as usize;

            if offset > bytes.len() {
                return None;
            }
            let positions = decode_delta_u32(&bytes[offset..], pos_count)?;
            offset += {
                // 估算 Delta 编码长度（较复杂，实际应记录长度）
                // 这里简化：先解码，不精确计算 offset
                let tmp = encode_delta_u32(&positions);
                tmp.len()
            };

            postings.push(Posting { doc_id, positions });
        }

        Some(PostingList {
            doc_count,
            postings,
        })
    }

    /// 获取所有 doc_id（不复制位置）
    pub fn doc_ids(&self) -> Vec<u64> {
        self.postings.iter().map(|p| p.doc_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posting_add() {
        let mut pl = PostingList::new();
        pl.add(1, 0);
        pl.add(1, 5);
        pl.add(2, 3);
        assert_eq!(pl.doc_count, 2);
        assert_eq!(pl.postings[0].positions, vec![0, 5]);
        assert_eq!(pl.postings[1].positions, vec![3]);
    }

    #[test]
    fn test_posting_merge() {
        let mut a = PostingList::new();
        a.add(1, 0);
        a.add(3, 10);

        let mut b = PostingList::new();
        b.add(1, 5);
        b.add(2, 3);

        a.merge(&b);
        assert_eq!(a.doc_count, 3);
        let p1 = a.postings.iter().find(|p| p.doc_id == 1).unwrap();
        assert_eq!(p1.positions, vec![0, 5]);
    }

    #[test]
    fn test_posting_serde_roundtrip() {
        let mut pl = PostingList::new();
        pl.add(100, 0);
        pl.add(100, 5);
        pl.add(200, 3);
        pl.add(300, 10);

        let bytes = pl.to_bytes();
        let decoded = PostingList::from_bytes(&bytes).unwrap();
        assert_eq!(pl.doc_count, decoded.doc_count);
        assert_eq!(pl.postings, decoded.postings);
    }
}
