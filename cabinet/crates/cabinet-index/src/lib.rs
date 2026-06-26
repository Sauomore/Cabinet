//! Cabinet Index 索引层
//!
//! 提供 B-tree 前缀索引、LSM 合并、VByte/Delta 压缩、三级记忆架构。

pub mod archive_index;
pub mod compress;
pub mod posting;
pub mod token_store;
pub mod working_memory;

pub use archive_index::{ArchiveIndex, FeatureDrawer, Hit};
pub use compress::{decode_delta, decode_delta_u32, decode_vbyte, encode_delta, encode_delta_u32, encode_vbyte};
pub use posting::{Posting, PostingList};
pub use token_store::{TokenRecord, TokenStore};
pub use working_memory::{MemorySnippet, WorkingMemory};
