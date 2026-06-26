use anyhow::Result;
use cabinet_hsh::perfect_hash::search_seed;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 离线构建聚类中心
/// 简化版：从语料统计词频，生成 mock 聚类中心文件
pub fn build_cluster_centers(corpus_path: &Path, output: &Path) -> Result<()> {
    println!("[build] 读取语料: {}", corpus_path.display());
    let text = fs::read_to_string(corpus_path)?;

    use cabinet_hsh::Encoder;
    let encoder = Encoder::new();
    let codes = encoder.encode(&text)?;

    // 按 feat 分组统计
    let mut feat_groups: HashMap<u8, Vec<String>> = HashMap::new();
    // 简化：这里只是统计词数，实际应使用 BERT 向量 + K-means
    println!("[build] 编码完成，{} tokens", codes.len());
    println!("[build] 生成聚类中心文件 (mock): {}", output.display());

    // 创建空的聚类中心文件作为占位（实际项目需接入真实 BERT 向量）
    use cabinet_hsh::cluster::{ClusterCenter, ClusterGroup, ClusterCenters};
    let mut groups = Vec::new();
    for feat in 0..16u8 {
        let centers = vec![
            ClusterCenter { id: 0, vector: vec![0.0; 768] },
            ClusterCenter { id: 1, vector: vec![1.0; 768] },
        ];
        groups.push(ClusterGroup { feat, centers });
    }
    let centers = ClusterCenters::new(groups);
    let bytes = centers.to_bytes();
    fs::write(output, bytes)?;

    println!("[build] 完成 → {} bytes", fs::metadata(output)?.len());
    Ok(())
}

/// 离线构建种子表
/// 对每个 (feat, sim) 簇搜索完美哈希种子
pub fn build_seed_table(cluster_map_path: &Path, output: &Path) -> Result<()> {
    println!("[build] 构建种子表: {}", output.display());

    // 读取聚类映射（简化：从语料统计词 → 簇分配）
    // 实际流程：对每个簇内词表搜索 seed
    let mut seeds = [0u8; 16 * 256];

    for feat in 0..16u8 {
        for sim in 0..=255u8 {
            // 模拟每个簇有 20~50 个词
            let words: Vec<String> = (0..30).map(|i| format!("word_{}_{}_{}", feat, sim, i)).collect();
            let (seed, _, _) = search_seed(&words);
            seeds[(feat as usize) * 256 + (sim as usize)] = seed;
        }
    }

    // 保存为二进制格式
    let mut buf = Vec::new();
    buf.extend_from_slice(&0xCAB1_0002u32.to_be_bytes()); // magic
    buf.extend_from_slice(&1u32.to_be_bytes());             // version
    buf.extend_from_slice(&seeds);                          // 4096 bytes

    fs::write(output, buf)?;
    println!("[build] 完成 → {} bytes", fs::metadata(output)?.len());
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: cabinet-build <corpus.txt> [output_dir]");
        std::process::exit(1);
    }

    let corpus = Path::new(&args[1]);
    let output_dir = if args.len() >= 3 {
        Path::new(&args[2])
    } else {
        Path::new("./assets")
    };

    fs::create_dir_all(output_dir)?;

    build_cluster_centers(corpus, &output_dir.join("cluster_centers.bin"))?;
    build_seed_table(corpus, &output_dir.join("seed_table.bin"))?;

    println!("\n[完成] 所有离线资源已构建到: {}", output_dir.display());
    Ok(())
}
