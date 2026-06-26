//! 聚类中心定义与相似码分配
//!
//! 运行时仅加载聚类中心（f32 数组），无需 BERT。
//! 新词通过欧氏距离归入最近簇。

/// 单个聚类中心（768-dim f32 向量，简化版可用更低维度）
pub struct ClusterCenter {
    pub id: u8,       // 相似码（簇 ID）
    pub vector: Vec<f32>,
}

impl ClusterCenter {
    /// 计算与目标向量的欧氏距离
    pub fn distance(&self, other: &[f32]) -> f32 {
        assert_eq!(self.vector.len(), other.len(), "维度不匹配");
        self.vector
            .iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt()
    }
}

/// 单个特征码下的所有聚类中心
pub struct ClusterGroup {
    pub feat: u8,
    pub centers: Vec<ClusterCenter>,
}

impl ClusterGroup {
    /// 查找最近簇，返回 (sim_id, distance)
    pub fn nearest(&self, vector: &[f32]) -> Option<(u8, f32)> {
        self.centers
            .iter()
            .map(|c| (c.id, c.distance(vector)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

/// 聚类中心集合：16 个特征码组
pub struct ClusterCenters {
    groups: Vec<ClusterGroup>,
}

impl ClusterCenters {
    pub fn new(groups: Vec<ClusterGroup>) -> Self {
        ClusterCenters { groups }
    }

    pub fn group(&self, feat: u8) -> Option<&ClusterGroup> {
        self.groups.iter().find(|g| g.feat == feat)
    }

    /// 分配相似码：查找最近簇，若距离 > τ 则返回 None（需创建新簇）
    pub fn assign_sim(&self, vector: &[f32], feat: u8, tau: f32) -> Option<u8> {
        let group = self.group(feat)?;
        let (sim, dist) = group.nearest(vector)?;
        if dist <= tau {
            Some(sim)
        } else {
            None
        }
    }

    /// 从二进制文件加载聚类中心
    /// 格式：magic(u32) + version(u32) + dim(u32) + num_groups(u32)
    ///       for each group: k_g(u32) + [f32 × k_g × dim]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("数据过短".to_string());
        }
        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != 0xCAB1_0001 {
            return Err(format!("magic 不匹配: 0x{:08X}", magic));
        }
        let _version = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let dim = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let num_groups = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;

        let mut offset = 16usize;
        let mut groups = Vec::with_capacity(num_groups);
        for _ in 0..num_groups {
            if offset + 4 > bytes.len() {
                return Err("组数据不足".to_string());
            }
            let k = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            let expected_size = k * dim * 4;
            if offset + expected_size > bytes.len() {
                return Err("中心向量数据不足".to_string());
            }

            let mut centers = Vec::with_capacity(k);
            for i in 0..k {
                let mut vector = Vec::with_capacity(dim);
                for d in 0..dim {
                    let base = offset + (i * dim + d) * 4;
                    vector.push(f32::from_be_bytes([
                        bytes[base],
                        bytes[base + 1],
                        bytes[base + 2],
                        bytes[base + 3],
                    ]));
                }
                centers.push(ClusterCenter {
                    id: i as u8,
                    vector,
                });
            }
            offset += expected_size;

            // feat 从 groups 顺序推导，假设按 0..15 顺序
            groups.push(ClusterGroup {
                feat: groups.len() as u8,
                centers,
            });
        }

        Ok(ClusterCenters::new(groups))
    }

    /// 序列化为二进制文件
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0xCAB1_0001u32.to_be_bytes()); // magic
        buf.extend_from_slice(&1u32.to_be_bytes());           // version
        let dim = self.groups.first().map(|g| g.centers.first().map(|c| c.vector.len()).unwrap_or(0)).unwrap_or(0);
        buf.extend_from_slice(&(dim as u32).to_be_bytes());
        buf.extend_from_slice(&(self.groups.len() as u32).to_be_bytes());

        for group in &self.groups {
            buf.extend_from_slice(&(group.centers.len() as u32).to_be_bytes());
            for center in &group.centers {
                for &v in &center.vector {
                    buf.extend_from_slice(&v.to_be_bytes());
                }
            }
        }
        buf
    }
}

/// 简化的词向量嵌入（MVP 阶段：用随机/固定映射替代真实 BERT）
/// v1.0 阶段替换为真实预训练向量
pub fn mock_embed(word: &str, dim: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    word.hash(&mut hasher);
    let seed = hasher.finish();
    let mut vec = Vec::with_capacity(dim);
    let mut state = seed;
    for _ in 0..dim {
        // xorshift64* 伪随机
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let val = ((state >> 32) & 0x7FFFFF) as f32 / 0x7FFFFF as f32; // [0, 1)
        vec.push(val * 2.0 - 1.0); // [-1, 1)
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_center_distance() {
        let c = ClusterCenter {
            id: 0,
            vector: vec![0.0, 1.0, 2.0],
        };
        let d = c.distance(&[0.0, 1.0, 2.0]);
        assert!(d < 0.001);
        let d2 = c.distance(&[1.0, 1.0, 2.0]);
        assert!((d2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cluster_group_nearest() {
        let g = ClusterGroup {
            feat: 0,
            centers: vec![
                ClusterCenter { id: 0, vector: vec![0.0, 0.0] },
                ClusterCenter { id: 1, vector: vec![1.0, 1.0] },
                ClusterCenter { id: 2, vector: vec![2.0, 2.0] },
            ],
        };
        let (sim, dist) = g.nearest(&[0.1, 0.1]).unwrap();
        assert_eq!(sim, 0);
        assert!(dist < 0.2);
    }

    #[test]
    fn test_centers_serde() {
        let groups = vec![
            ClusterGroup {
                feat: 0,
                centers: vec![
                    ClusterCenter { id: 0, vector: vec![0.1, 0.2, 0.3] },
                    ClusterCenter { id: 1, vector: vec![0.4, 0.5, 0.6] },
                ],
            },
            ClusterGroup {
                feat: 1,
                centers: vec![
                    ClusterCenter { id: 0, vector: vec![0.7, 0.8, 0.9] },
                ],
            },
        ];
        let centers = ClusterCenters::new(groups);
        let bytes = centers.to_bytes();
        let decoded = ClusterCenters::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.groups.len(), 2);
        assert_eq!(decoded.groups[0].centers.len(), 2);
        assert_eq!(decoded.groups[1].centers.len(), 1);
    }

    #[test]
    fn test_mock_embed() {
        let v1 = mock_embed("hello", 768);
        let v2 = mock_embed("hello", 768);
        let v3 = mock_embed("world", 768);
        assert_eq!(v1, v2); // 同一词产生相同向量
        assert_ne!(v1, v3); // 不同词不同向量
        assert_eq!(v1.len(), 768);
    }
}
