use cabinet_core::{Config, Memory, QueryOpts};
use std::time::{Duration, Instant};

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("bench.db"))).unwrap();

    // 准备测试数据
    let texts: Vec<String> = (0..1000)
        .map(|i| format!(
            "这是第{}条测试文档，包含各种词汇如会议、准备、测试、数据、社区、邻里、借用、归还等。",
            i
        ))
        .collect();

    // 插入基准
    let start = Instant::now();
    let ids = mem.insert_batch(&texts).unwrap();
    let insert_time = start.elapsed();
    println!(
        "[Insert] {} docs in {:?} ({:.1} docs/s)",
        ids.len(),
        insert_time,
        ids.len() as f64 / insert_time.as_secs_f64()
    );

    // 查询基准
    let queries = vec!["会议准备", "社区邻里", "借用归还", "测试数据"];
    for q in queries {
        let start = Instant::now();
        let mut total = Duration::ZERO;
        for _ in 0..100 {
            let s = Instant::now();
            let _ = mem.query(q, QueryOpts::new().top_k(10)).unwrap();
            total += s.elapsed();
        }
        let avg = total / 100;
        println!(
            "[Query] \"{}\" => avg {:.3} ms (100 runs, total {:?})",
            q,
            avg.as_secs_f64() * 1000.0,
            total
        );
    }

    mem.close().unwrap();
    println!("[Done] Benchmark complete.");
}
