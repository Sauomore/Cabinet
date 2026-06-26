use cabinet_core::{Config, Memory, QueryOpts};

fn main() -> anyhow::Result<()> {
    let mut mem = Memory::open(Config::new("./agent_memory.db"))?;

    // 插入示例
    let docs = vec![
        "用户明天下午3点开会，准备PPT。",
        "用户喜欢听管弦乐。",
        "5号楼邻居有梯子，平时放在车库。",
        "上周 user_456 借了梯子给 3 号楼，周三还的。",
    ];

    for doc in &docs {
        let id = mem.insert(doc)?;
        println!("[insert] doc_id={}: {}", id, doc);
    }

    // 查询
    let queries = vec!["会议准备", "借梯子", "社区活动"];
    for q in queries {
        let results = mem.query(q, QueryOpts::new().top_k(3))?;
        println!("\n[query] \"{}\" → {} results", q, results.len());
        for r in &results {
            let level = match r.match_level {
                4 => "EXACT",
                3 => "CLUSTER",
                2 => "CATEGORY",
                1 => "RELATED",
                _ => "UNKNOWN",
            };
            println!("  [{}] score={:.3} doc_id={}", level, r.score, r.doc_id);
        }
    }

    mem.close()?;
    Ok(())
}
