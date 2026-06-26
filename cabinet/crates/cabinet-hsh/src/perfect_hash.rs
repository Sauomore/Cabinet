//! 准完美哈希实现
//!
//! 对每个簇 (feat, sim) 搜索种子 s ∈ [0, 255]，使得：
//! abs(wi) = (BKDR(wi) XOR s) mod 256
//! 对所有词 wi 互不相同。
//!
//! 种子搜索期望复杂度：O(125) 次尝试（n=50 时）

use std::collections::HashMap;

/// BKDR 字符串哈希函数
pub fn bkdr_hash(s: &str) -> u64 {
    let seed: u64 = 131; // 131, 1313, 13131, 131313 等均可
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(seed).wrapping_add(byte as u64);
    }
    hash
}

/// 计算候选绝对码
/// abs = (BKDR(word) XOR seed) mod 256
pub fn compute_abs(word: &str, seed: u8) -> u8 {
    let h = bkdr_hash(word);
    ((h ^ (seed as u64)) % 256) as u8
}

/// 单个簇的完美哈希种子搜索
/// 返回 (seed, word_to_abs 映射表)
/// 若无法找到完美哈希，返回最佳种子及映射表（带溢出标记）
pub fn search_seed(words: &[String]) -> (u8, HashMap<String, u8>, Vec<u8>) {
    let n = words.len();
    if n == 0 {
        return (0, HashMap::new(), Vec::new());
    }
    if n > 256 {
        // 超出 256 个桶，无法完美哈希，必须降级
        return search_best_seed(words);
    }

    // 暴力搜索种子
    for seed in 0..=255u8 {
        let mut used = [false; 256];
        let mut collision = false;
        for word in words {
            let abs = compute_abs(word, seed);
            if used[abs as usize] {
                collision = true;
                break;
            }
            used[abs as usize] = true;
        }
        if !collision {
            let mut map = HashMap::with_capacity(n);
            for word in words {
                let abs = compute_abs(word, seed);
                map.insert(word.clone(), abs);
            }
            let abs_list: Vec<u8> = words.iter().map(|w| map[w]).collect();
            return (seed, map, abs_list);
        }
    }

    // 未找到完美哈希，降级到最佳种子（碰撞最少）
    search_best_seed(words)
}

/// 搜索碰撞最少的种子（降级策略）
fn search_best_seed(words: &[String]) -> (u8, HashMap<String, u8>, Vec<u8>) {
    let mut best_seed = 0u8;
    let mut min_collision = usize::MAX;
    let mut best_map = HashMap::new();

    for seed in 0..=255u8 {
        let mut counts = [0u16; 256];
        for word in words {
            let abs = compute_abs(word, seed);
            counts[abs as usize] += 1;
        }
        let collisions: usize = counts.iter().map(|&c| if c > 1 { (c - 1) as usize } else { 0 }).sum();
        if collisions < min_collision {
            min_collision = collisions;
            best_seed = seed;
            let mut map = HashMap::with_capacity(words.len());
            for word in words {
                let abs = compute_abs(word, seed);
                map.insert(word.clone(), abs);
            }
            best_map = map;
            if collisions == 0 {
                break; // 找到完美哈希
            }
        }
    }

    let abs_list: Vec<u8> = words.iter().map(|w| best_map[w]).collect();
    (best_seed, best_map, abs_list)
}

/// 种子表：16 (feat) × 256 (sim) 的种子
/// 扁平数组索引：seed_table[feat * 256 + sim]
pub struct SeedTable {
    seeds: [u8; 16 * 256],
    // 每个簇的 word → abs 映射表（按需加载）
    mappings: [Option<HashMap<String, u8>>; 16 * 256],
}

impl SeedTable {
    pub fn new() -> Self {
        SeedTable {
            seeds: [0u8; 16 * 256],
            mappings: std::array::from_fn(|_| None),
        }
    }

    fn idx(feat: u8, sim: u8) -> usize {
        (feat as usize) * 256 + (sim as usize)
    }

    pub fn set_seed(&mut self, feat: u8, sim: u8, seed: u8) {
        self.seeds[Self::idx(feat, sim)] = seed;
    }

    pub fn get_seed(&self, feat: u8, sim: u8) -> u8 {
        self.seeds[Self::idx(feat, sim)]
    }

    pub fn set_mapping(&mut self, feat: u8, sim: u8, mapping: HashMap<String, u8>) {
        self.mappings[Self::idx(feat, sim)] = Some(mapping);
    }

    pub fn get_abs(&self, word: &str, feat: u8, sim: u8) -> Option<u8> {
        self.mappings[Self::idx(feat, sim)]
            .as_ref()
            .and_then(|m| m.get(word).copied())
    }

    /// 计算绝对码（运行时：先查映射表，未命中则计算候选）
    pub fn compute_abs(&self, word: &str, feat: u8, sim: u8) -> u8 {
        if let Some(abs) = self.get_abs(word, feat, sim) {
            return abs;
        }
        let seed = self.get_seed(feat, sim);
        super::compute_abs(word, seed)
    }

    /// 序列化为二进制格式
    /// 格式：magic(u32) + version(u32) + seeds([u8; 4096]) + 映射表数量 + 映射表数据
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0xCAB1_0002u32.to_be_bytes()); // magic
        buf.extend_from_slice(&1u32.to_be_bytes());             // version
        buf.extend_from_slice(&self.seeds);                     // 4096 bytes
        // TODO: 映射表序列化（较复杂，留待后续实现）
        buf
    }
}

impl Default for SeedTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bkdr_hash() {
        let h1 = bkdr_hash("hello");
        let h2 = bkdr_hash("hello");
        let h3 = bkdr_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_compute_abs() {
        let abs = compute_abs("测试", 42);
        assert!(abs <= 255);
    }

    #[test]
    fn test_search_seed_perfect() {
        let words: Vec<String> = (0..20).map(|i| format!("word_{}", i)).collect();
        let (seed, map, _) = search_seed(&words);
        // 检查是否无碰撞
        let mut used = [false; 256];
        for word in &words {
            let abs = map[word];
            assert!(!used[abs as usize], "碰撞！word={} abs={}", word, abs);
            used[abs as usize] = true;
        }
        println!("seed={}, perfect for {} words", seed, words.len());
    }

    #[test]
    fn test_search_seed_overflow() {
        // 超过 256 个词，必然碰撞
        let words: Vec<String> = (0..300).map(|i| format!("word_{}", i)).collect();
        let (seed, map, _) = search_seed(&words);
        assert!(map.len() == 300);
        // 统计碰撞
        let mut counts = [0u16; 256];
        for abs in map.values() {
            counts[*abs as usize] += 1;
        }
        let collisions: usize = counts.iter().map(|&c| if c > 1 { (c - 1) as usize } else { 0 }).sum();
        println!("seed={}, collisions={} for 300 words", seed, collisions);
        assert!(collisions > 0); // 必然有碰撞
    }

    #[test]
    fn test_seed_table() {
        let mut table = SeedTable::new();
        table.set_seed(0x0, 0x42, 123);
        assert_eq!(table.get_seed(0x0, 0x42), 123);

        let mut mapping = HashMap::new();
        mapping.insert("测试词".to_string(), 0xAB);
        table.set_mapping(0x0, 0x42, mapping);

        assert_eq!(table.get_abs("测试词", 0x0, 0x42), Some(0xAB));
        assert_eq!(table.get_abs("未知词", 0x0, 0x42), None);
    }
}
