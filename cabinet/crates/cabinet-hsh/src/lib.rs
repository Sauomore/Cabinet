//! Cabinet HSH 编码层
//!
//! 20-bit 层次语义哈希（Hierarchical Semantic Hashing）的纯计算实现。
//! 零 IO、零线程、零网络，仅操作内存中的字符串和整数。

pub mod cluster;
pub mod encoder;
pub mod error;
pub mod hsh_code;
pub mod perfect_hash;
pub mod pos_map;

pub use cluster::{ClusterCenter, ClusterGroup, ClusterCenters, mock_embed};
pub use encoder::{Encoder, EncoderConfig};
pub use error::EncodeError;
pub use hsh_code::HSHCode;
pub use perfect_hash::{SeedTable, bkdr_hash, compute_abs, search_seed};
pub use pos_map::{feat_name, pos_to_feat, FeatureCode};
