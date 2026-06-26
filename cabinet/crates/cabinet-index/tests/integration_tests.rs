use cabinet_index::{ArchiveIndex, TokenStore, WorkingMemory, MemorySnippet};
use cabinet_hsh::HSHCode;

#[test]
fn test_token_store_threshold() {
    let mut store = TokenStore::new(5);
    
    for i in 1..=5 {
        let id = store.insert(vec![HSHCode::new(0, i as u8, i as u8)]);
        assert_eq!(id, i as u64);
    }
    
    assert!(store.should_merge());
    
    let drained = store.drain();
    assert_eq!(drained.len(), 5);
    assert!(store.is_empty());
    assert!(!store.should_merge());
}

#[test]
fn test_archive_index_multiple_docs() {
    let mut index = ArchiveIndex::new();
    
    let docs = vec![
        (1u64, vec![HSHCode::new(0x0, 0x15, 0x01), HSHCode::new(0x1, 0x42, 0x02)]),
        (2u64, vec![HSHCode::new(0x0, 0x15, 0x03), HSHCode::new(0x2, 0x20, 0x04)]),
        (3u64, vec![HSHCode::new(0x0, 0x15, 0x01), HSHCode::new(0x1, 0x42, 0x05)]),
    ];
    
    for (doc_id, hsh_seq) in &docs {
        index.merge_from_hsh_seq(*doc_id, hsh_seq);
    }
    
    let hits = index.query(HSHCode::new(0x0, 0x15, 0x01), &[]);
    assert!(!hits.is_empty());
    
    let doc_ids: Vec<_> = hits.iter().map(|h| h.doc_id).collect();
    assert!(doc_ids.contains(&1));
    assert!(doc_ids.contains(&3));
}

#[test]
fn test_working_memory_lru() {
    let mut wm = WorkingMemory::new(3);
    
    let h1 = HSHCode::new(0, 1, 2);
    let h2 = HSHCode::new(0, 1, 3);
    let h3 = HSHCode::new(0, 1, 4);
    let h4 = HSHCode::new(0, 1, 5);
    
    wm.load(h1, MemorySnippet { doc_id: 1, text: "a".to_string() });
    wm.load(h2, MemorySnippet { doc_id: 2, text: "b".to_string() });
    wm.load(h3, MemorySnippet { doc_id: 3, text: "c".to_string() });
    
    wm.query(h1);
    wm.query(h1);
    wm.query(h2);
    
    wm.load(h4, MemorySnippet { doc_id: 4, text: "d".to_string() });
    
    assert_eq!(wm.len(), 3);
    assert!(wm.query(h3).is_none());
    assert!(wm.query(h1).is_some());
    assert!(wm.query(h2).is_some());
    assert!(wm.query(h4).is_some());
}

#[test]
fn test_posting_list_merge() {
    use cabinet_index::PostingList;
    
    let mut a = PostingList::new();
    a.add(1, 0);
    a.add(1, 5);
    a.add(3, 10);
    
    let mut b = PostingList::new();
    b.add(1, 3);
    b.add(2, 0);
    b.add(3, 15);
    
    a.merge(&b);
    
    let p1 = a.postings.iter().find(|p| p.doc_id == 1).unwrap();
    assert_eq!(p1.positions, vec![0, 3, 5]);
    
    let p2 = a.postings.iter().find(|p| p.doc_id == 2).unwrap();
    assert_eq!(p2.positions, vec![0]);
    
    let p3 = a.postings.iter().find(|p| p.doc_id == 3).unwrap();
    assert_eq!(p3.positions, vec![10, 15]);
}
