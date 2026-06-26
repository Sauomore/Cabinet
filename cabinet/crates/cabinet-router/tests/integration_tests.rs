use cabinet_router::Router;
use cabinet_hsh::HSHCode;

#[test]
fn test_router_related_comprehensive() {
    let router = Router::new();
    
    let noun_related = router.get_related(0x0);
    assert!(noun_related.iter().any(|a| a.feat == 0x1));
    assert!(noun_related.iter().any(|a| a.feat == 0x2));
    
    let verb_related = router.get_related(0x1);
    assert!(verb_related.iter().any(|a| a.feat == 0x0));
    assert!(verb_related.iter().any(|a| a.feat == 0x3));
    
    let common_related = router.get_related(0xE);
    assert!(common_related.len() > 5);
}

#[test]
fn test_router_hsh_integration() {
    let router = Router::new();
    let hsh = HSHCode::new(0x0, 0x15, 0x01);
    
    let related = router.related_feats_for_hsh(hsh);
    
    assert!(!related.is_empty());
    
    for (_, weight) in &related {
        assert!(*weight > 0.0 && *weight <= 1.0);
    }
}
