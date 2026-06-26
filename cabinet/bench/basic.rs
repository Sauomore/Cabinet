use cabinet_core::{Config, Memory, QueryOpts};
use std::time::Instant;

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("bench.db"))).unwrap();

    // 准备测试数据
    let texts: Vec<String> = (0..1000)
        .map(|i| format!("这是第 {} 条测试文档，包含各种词汇如会议、准备、测试、数据等。", i))
        .collect();

    let start = Instant::now();
    let ids = mem.insert_batch(&texts).unwrap();
    let insert_time = start.elapsed();
    println!("插入 {} 条文档: {:?} (平均 {:.2} μs/条)", 
        ids.len(), insert_time, insert_time.as_micros() as f64 / ids.len() as f64);

    let start = Instant::now();
    for _ in 0..100 {
        let _ = mem.query("会议准备", QueryOpts::new().top_k(10)).unwrap();
    }
    let query_time = start.elapsed();
    println!("100 次查询: {:?} (平均 {:.2} ms/次)", 
        query_time, query_time.as_millis() as f64 / 100.0);

    mem.close().unwrap();
}
