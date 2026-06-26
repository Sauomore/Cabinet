//! 路由层：RelRouter 特征码关联权重
//!
//! MVP 阶段使用硬编码关联矩阵，v0.5 后引入轻量 MLP。

use cabinet_hsh::HSHCode;

/// 关联权重（特征码 → 相关特征码 + 权重）
#[derive(Debug, Clone, PartialEq)]
pub struct Association {
    pub feat: u8,
    pub weight: f32,
}

/// Router：特征码关联路由器
/// MVP 冷启动：硬编码关联矩阵
pub struct Router {
    hard_rules: [[f32; 16]; 16],
    neural: Option<MLPWeights>, // v0.5 后使用
}

/// MLP 权重（v0.5 设计）
pub struct MLPWeights {
    pub embedding: Vec<f32>,     // 4096 × 32
    pub layer1: Vec<f32>,        // 32 × 64
    pub layer2: Vec<f32>,        // 64 × 16
    pub prior: [[f32; 16]; 16], // 硬编码偏置
}

impl Router {
    /// 创建默认 Router（硬编码关联规则）
    pub fn new() -> Self {
        let mut rules = [[0.0f32; 16]; 16];

        // 名词(0x0) 关联 动词(0x1, 0.8), 形容词(0x2, 0.6), 时间(0xA, 0.4)
        rules[0x0][0x1] = 0.8;
        rules[0x0][0x2] = 0.6;
        rules[0x0][0xA] = 0.4;

        // 动词(0x1) 关联 名词(0x0, 0.8), 副词(0x3, 0.5), 形容词(0x2, 0.3)
        rules[0x1][0x0] = 0.8;
        rules[0x1][0x3] = 0.5;
        rules[0x1][0x2] = 0.3;

        // 形容词(0x2) 关联 名词(0x0, 0.7), 动词(0x1, 0.4)
        rules[0x2][0x0] = 0.7;
        rules[0x2][0x1] = 0.4;

        // 副词(0x3) 关联 动词(0x1, 0.7), 形容词(0x2, 0.5)
        rules[0x3][0x1] = 0.7;
        rules[0x3][0x2] = 0.5;

        // 代词(0x4) 关联 名词(0x0, 0.6), 动词(0x1, 0.5)
        rules[0x4][0x0] = 0.6;
        rules[0x4][0x1] = 0.5;

        // 时间(0xA) 关联 名词(0x0, 0.5), 动词(0x1, 0.6)
        rules[0xA][0x0] = 0.5;
        rules[0xA][0x1] = 0.6;

        // 常用词(0xE) 关联 所有其他（权重较低）
        for i in 0..16 {
            if i != 0xE {
                rules[0xE][i] = 0.3;
            }
        }

        Router {
            hard_rules: rules,
            neural: None,
        }
    }

    /// 获取与给定特征码相关的所有特征码（权重 > 0.1）
    pub fn get_related(&self, feat: u8) -> Vec<Association> {
        self.hard_rules[feat as usize]
            .iter()
            .enumerate()
            .filter(|(_, w)| **w > 0.1)
            .map(|(f, w)| Association {
                feat: f as u8,
                weight: *w,
            })
            .collect()
    }

    /// 为 HSH 查询生成相关特征码列表（用于检索时的 match level 1）
    pub fn related_feats_for_hsh(&self, hsh: HSHCode) -> Vec<(u8, f32)> {
        self.get_related(hsh.feat())
            .into_iter()
            .map(|a| (a.feat, a.weight))
            .collect()
    }

    /// 加载 MLP 权重（v0.5 后）
    pub fn load_mlp(&mut self, _weights: MLPWeights) {
        // TODO: v0.5 实现
        // 若 ONNX 加载失败，回退到硬编码规则
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_related() {
        let router = Router::new();
        let related = router.get_related(0x0); // 名词
        assert!(!related.is_empty());
        // 名词应关联动词
        assert!(related.iter().any(|a| a.feat == 0x1));
        // 动词权重 0.8
        let verb = related.iter().find(|a| a.feat == 0x1).unwrap();
        assert!((verb.weight - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_router_symmetry() {
        let router = Router::new();
        // 名词关联动词权重 0.8
        let n_v = router.get_related(0x0).iter().find(|a| a.feat == 0x1).map(|a| a.weight);
        // 动词关联名词权重 0.8
        let v_n = router.get_related(0x1).iter().find(|a| a.feat == 0x0).map(|a| a.weight);
        assert_eq!(n_v, v_n);
    }
}
