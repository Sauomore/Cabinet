//! Archive Index：Layer 2，核心倒排索引结构
//!
//! 16 个 Feature Drawer，每个内部按 (sim, abs) B-tree 索引。
//! 支持四级匹配（exact/cluster/category/related）和 LSM 合并。

use cabinet_hsh::HSHCode;
use crate::posting::{PostingList, Posting};
use std::collections::BTreeMap;

/// B-tree 键：高 8-bit sim + 低 8-bit abs，合并为 u16
pub type DrawerKey = u16;

/// 特征抽屉：同一 feat 下所有 sim×abs 的索引
#[derive(Debug, Clone, Default)]
pub struct FeatureDrawer {
    pub feat: u8,
    pub tree: BTreeMap<DrawerKey, PostingList>,
}

impl FeatureDrawer {
    pub fn new(feat: u8) -> Self {
        FeatureDrawer {
            feat,
            tree: BTreeMap::new(),
        }
    }

    /// 插入 posting（追加）
    pub fn insert(&mut self, sim: u8, abs: u8, doc_id: u64, position: u32) {
        let key = ((sim as u16) << 8) | (abs as u16);
        self.tree
            .entry(key)
            .or_insert_with(PostingList::new)
            .add(doc_id, position);
    }

    /// 精确匹配：key = (sim, abs)
    pub fn exact_query(&self, sim: u8, abs: u8) -> Option<&PostingList> {
        let key = ((sim as u16) << 8) | (abs as u16);
        self.tree.get(&key)
    }

    /// 同簇前缀扫描：sim 固定，abs 遍历 0x00..0xFF
    /// key range: [sim<<8, (sim+1)<<8)
    pub fn sim_prefix_scan(&self, sim: u8) -> Vec<&PostingList> {
        let start = (sim as u16) << 8;
        let end = ((sim as u16) + 1) << 8;
        self.tree
            .range(start..end)
            .map(|(_, pl)| pl)
            .collect()
    }

    /// 同类扫描：整个 feat 抽屉（范围扫描）
    pub fn all(&self) -> Vec<&PostingList> {
        self.tree.values().collect()
    }
}

/// 检索命中项（带权重）
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub doc_id: u64,
    pub position: u32,
    pub match_level: u8, // 4=exact, 3=cluster, 2=category, 1=related
    pub score: f32,
    pub weight: f32, // 关联权重（Router 提供）
}

impl Hit {
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self.score *= weight;
        self
    }
}

/// Archive Index：16 个抽屉组成
#[derive(Debug, Clone, Default)]
pub struct ArchiveIndex {
    drawers: [FeatureDrawer; 16],
}

impl ArchiveIndex {
    pub fn new() -> Self {
        ArchiveIndex {
            drawers: std::array::from_fn(|i| FeatureDrawer::new(i as u8)),
        }
    }

    /// 从 TokenRecord 批量构建/合并索引
    pub fn merge_from_hsh_seq(&mut self, doc_id: u64, hsh_seq: &[HSHCode]) {
        for (pos, hsh) in hsh_seq.iter().enumerate() {
            let feat = hsh.feat();
            let sim = hsh.sim();
            let abs = hsh.abs();
            self.drawers[feat as usize].insert(sim, abs, doc_id, pos as u32);
        }
    }

    /// 四级匹配检索
    /// 返回未排序的 Hit 列表（需后续聚合排序）
    pub fn query(&self, hsh: HSHCode, related_feats: &[(u8, f32)]) -> Vec<Hit> {
        let mut hits = Vec::new();
        let feat = hsh.feat() as usize;
        let sim = hsh.sim();
        let abs = hsh.abs();

        // 1. 精确匹配（match level 4）
        if let Some(pl) = self.drawers[feat].exact_query(sim, abs) {
            for posting in &pl.postings {
                for &pos in &posting.positions {
                    hits.push(Hit {
                        doc_id: posting.doc_id,
                        position: pos,
                        match_level: 4,
                        score: 1.0,
                        weight: 1.0,
                    });
                }
            }
        }

        // 2. 同簇匹配（match level 3）
        for pl in self.drawers[feat].sim_prefix_scan(sim) {
            for posting in &pl.postings {
                for &pos in &posting.positions {
                    hits.push(Hit {
                        doc_id: posting.doc_id,
                        position: pos,
                        match_level: 3,
                        score: 0.7,
                        weight: 1.0,
                    });
                }
            }
        }

        // 3. 同类匹配（match level 2）— 已在同簇扫描中隐含（feat 相同），
        // 但这里显式扫描整个 drawer 以捕获不同 sim 的 posting
        for pl in self.drawers[feat].all() {
            for posting in &pl.postings {
                for &pos in &posting.positions {
                    hits.push(Hit {
                        doc_id: posting.doc_id,
                        position: pos,
                        match_level: 2,
                        score: 0.4,
                        weight: 1.0,
                    });
                }
            }
        }

        // 4. 相关类别（match level 1）
        for &(rel_feat, weight) in related_feats {
            if rel_feat as usize == feat {
                continue; // 跳过同类（已处理）
            }
            for pl in self.drawers[rel_feat as usize].sim_prefix_scan(sim) {
                for posting in &pl.postings {
                    for &pos in &posting.positions {
                        hits.push(Hit {
                            doc_id: posting.doc_id,
                            position: pos,
                            match_level: 1,
                            score: 0.2 * weight,
                            weight,
                        });
                    }
                }
            }
        }

        hits
    }

    /// 聚合按文档排序：计算 score(d, Q) = Σ max ω · pos_prox
    pub fn aggregate_and_rank(hits: Vec<Hit>, top_k: usize) -> Vec<Hit> {
        // 按 (doc_id, match_level) 聚合
        use std::collections::HashMap;
        let mut doc_scores: HashMap<u64, Vec<Hit>> = HashMap::new();
        for hit in hits {
            doc_scores.entry(hit.doc_id).or_default().push(hit);
        }

        let mut aggregated: Vec<Hit> = doc_scores
            .into_iter()
            .map(|(doc_id, mut doc_hits)| {
                // 同一文档内取最高 level 的 hit 作为代表
                doc_hits.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let best = doc_hits.into_iter().next().unwrap();
                Hit {
                    doc_id,
                    position: best.position,
                    match_level: best.match_level,
                    score: best.score,
                    weight: best.weight,
                }
            })
            .collect();

        aggregated.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        aggregated.truncate(top_k);
        aggregated
    }

    /// 获取 drawer 的引用
    pub fn drawer(&self, feat: u8) -> &FeatureDrawer {
        &self.drawers[feat as usize]
    }

    pub fn drawer_mut(&mut self, feat: u8) -> &mut FeatureDrawer {
        &mut self.drawers[feat as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cabinet_hsh::HSHCode;

    #[test]
    fn test_archive_index_insert_and_query() {
        let mut index = ArchiveIndex::new();
        let doc_id = 1u64;
        let hsh_seq = vec![
            HSHCode::new(0x0, 0x15, 0x01), // 名词 "会议"
            HSHCode::new(0x1, 0x42, 0x02), // 动词 "准备"
            HSHCode::new(0x0, 0x15, 0x03), // 名词 "PPT"
        ];
        index.merge_from_hsh_seq(doc_id, &hsh_seq);

        // 精确匹配
        let hits = index.query(HSHCode::new(0x0, 0x15, 0x01), &[]);
        assert!(!hits.is_empty());
        let exact = hits.iter().find(|h| h.match_level == 4);
        assert!(exact.is_some());
        assert_eq!(exact.unwrap().doc_id, 1);
    }

    #[test]
    fn test_archive_index_merge() {
        let mut index = ArchiveIndex::new();
        let seq1 = vec![HSHCode::new(0x0, 0x1, 0x2)];
        let seq2 = vec![HSHCode::new(0x0, 0x1, 0x2)];
        index.merge_from_hsh_seq(1, &seq1);
        index.merge_from_hsh_seq(2, &seq2);

        let pl = index.drawer(0x0).exact_query(0x1, 0x2).unwrap();
        assert_eq!(pl.doc_count, 2);
    }

    #[test]
    fn test_ranking() {
        let hits = vec![
            Hit { doc_id: 1, position: 0, match_level: 4, score: 1.0, weight: 1.0 },
            Hit { doc_id: 1, position: 1, match_level: 3, score: 0.7, weight: 1.0 },
            Hit { doc_id: 2, position: 0, match_level: 4, score: 1.0, weight: 1.0 },
            Hit { doc_id: 3, position: 0, match_level: 2, score: 0.4, weight: 1.0 },
        ];
        let ranked = ArchiveIndex::aggregate_and_rank(hits, 2);
        assert_eq!(ranked.len(), 2);
        // doc 1 和 2 都应出现（score 相同，排序不稳定但都应存在）
        let ids: Vec<_> = ranked.iter().map(|h| h.doc_id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }
}
