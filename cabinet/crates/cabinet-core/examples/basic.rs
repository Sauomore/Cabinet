use cabinet_core::{Config, Memory, QueryOpts};

fn main() -> anyhow::Result<()> {
    let mut mem = Memory::open(Config::new("./agent_memory.db"))?;
    
    // 插入记忆
    let doc_id = mem.insert("用户明天下午3点开会，准备PPT。")?;
    println!("插入文档 ID: {}", doc_id);
    
    mem.insert("用户喜欢听管弦乐。")?;
    
    // 检索记忆
    let results = mem.query("会议准备", QueryOpts::new().top_k(5))?;
    for hit in results {
        println!("doc={}, pos={}, match_level={}, score={:.3}",
            hit.doc_id, hit.position, hit.match_level, hit.score);
        
        if hit.match_level >= 3 {
            let text = mem.decode(&hit);
            println!("  text: {:?}", text);
        }
    }
    
    // 逻辑运算
    let meetings = mem.scan_bucket(0x0, 0x15);
    let preps = mem.scan_bucket(0x1, 0x42);
    println!("同时涉及 '会议' 和 '准备' 的文档: {:?}", 
        meetings.iter().filter(|id| preps.contains(id)).collect::<Vec<_>>());
    
    mem.close()?;
    Ok(())
}
