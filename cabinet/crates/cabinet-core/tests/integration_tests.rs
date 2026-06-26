use cabinet_core::{Config, Memory, QueryOpts};

#[test]
fn test_memory_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("test.db"))).unwrap();
    
    let id1 = mem.insert("用户明天下午3点开会，准备PPT。").unwrap();
    let id2 = mem.insert("用户喜欢听管弦乐。").unwrap();
    let id3 = mem.insert("准备明天的会议资料。").unwrap();
    
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    
    let results = mem.query("会议准备", QueryOpts::new().top_k(10)).unwrap();
    assert!(!results.is_empty());
    
    let doc_ids: Vec<_> = results.iter().map(|r| r.doc_id).collect();
    assert!(doc_ids.contains(&1) || doc_ids.contains(&3));
    
    mem.close().unwrap();
}

#[test]
fn test_memory_batch_insert() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("batch.db"))).unwrap();
    
    let texts: Vec<String> = (1..=100)
        .map(|i| format!("这是第{}条测试文档，包含会议、准备、测试等词汇。", i))
        .collect();
    
    let ids = mem.insert_batch(&texts).unwrap();
    assert_eq!(ids.len(), 100);
    
    let results = mem.query("会议准备", QueryOpts::new().top_k(5)).unwrap();
    assert!(!results.is_empty());
    
    mem.close().unwrap();
}

#[test]
fn test_memory_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("snap.db"))).unwrap();
    
    mem.insert("测试快照功能。").unwrap();
    
    let backup_path = dir.path().join("backup.db");
    mem.snapshot(&backup_path).unwrap();
    
    assert!(backup_path.exists());
    
    mem.close().unwrap();
}

#[test]
fn test_memory_decode() {
    let dir = tempfile::tempdir().unwrap();
    let mut mem = Memory::open(Config::new(dir.path().join("decode.db"))).unwrap();
    
    let text = "这是一个可解码的测试文本。";
    mem.insert(text).unwrap();
    
    let results = mem.query("测试", QueryOpts::new().top_k(1)).unwrap();
    assert!(!results.is_empty());
    
    let decoded = mem.decode(&results[0]);
    assert!(decoded.is_some());
    
    mem.close().unwrap();
}
