use cabinet_hsh::{Encoder, HSHCode, pos_map::pos_to_feat, FeatureCode};

#[test]
fn test_pos_map_comprehensive() {
    let test_cases = vec![
        ("n", FeatureCode::NOUN),
        ("v", FeatureCode::VERB),
        ("vd", FeatureCode::VERB),
        ("a", FeatureCode::ADJ),
        ("ad", FeatureCode::ADJ),
        ("d", FeatureCode::ADV),
        ("r", FeatureCode::PRONOUN),
        ("p", FeatureCode::PREP),
        ("c", FeatureCode::CONJ),
        ("u", FeatureCode::AUX),
        ("m", FeatureCode::NUM),
        ("q", FeatureCode::MEASURE),
        ("t", FeatureCode::TIME),
        ("f", FeatureCode::LOC),
        ("w", FeatureCode::PUNCT),
        ("x", FeatureCode::STRING),
        ("unknown", FeatureCode::FALLBACK),
    ];

    for (tag, expected) in test_cases {
        let result = pos_to_feat(tag).unwrap();
        assert_eq!(result, expected, "POS tag '{}' should map to {:?}", tag, expected);
    }
}

#[test]
fn test_hsh_boundary_values() {
    let min = HSHCode::new(0, 0, 0);
    assert_eq!(min.raw(), 0);
    assert_eq!(min.feat(), 0);
    assert_eq!(min.sim(), 0);
    assert_eq!(min.abs(), 0);

    let max = HSHCode::new(0x0F, 0xFF, 0xFF);
    assert_eq!(max.raw(), 0x0F_FFFF);
    assert!(max.raw() < (1 << 20));
}

#[test]
fn test_hsh_bytes_consistency() {
    let code = HSHCode::new(0x0A, 0x42, 0xF0);
    for _ in 0..100 {
        let bytes = code.to_bytes();
        let decoded = HSHCode::from_bytes(bytes);
        assert_eq!(code, decoded);
    }
}

#[test]
fn test_encode_special_characters() {
    let encoder = Encoder::new();
    let texts = vec![
        "12345",
        "Hello World",
        "https://example.com",
        "email@domain.com",
        "😀🎉👍",
    ];

    for text in texts {
        let codes = encoder.encode(text).unwrap();
        assert!(codes.iter().all(|c| c.raw() < (1 << 20)));
    }
}

#[test]
fn test_encode_empty_and_whitespace() {
    let encoder = Encoder::new();
    
    let empty = encoder.encode("").unwrap();
    assert!(empty.is_empty());
    
    let whitespace = encoder.encode("   \t\n  ").unwrap();
    assert!(whitespace.len() <= 5);
}

#[test]
fn test_common_word_promotion_comprehensive() {
    let mut encoder = Encoder::new();
    
    encoder.add_common_word("测试");
    encoder.add_common_word("示例");
    encoder.add_common_word("数据");
    
    let codes = encoder.encode("测试示例数据").unwrap();
    
    let has_promoted = codes.iter().any(|c| c.feat() == 0x0E);
    assert!(has_promoted, "至少一个词应被晋升到常用词类别");
    
    encoder.remove_common_word("测试");
    let codes_after = encoder.encode("测试").unwrap();
    let no_longer_promoted = !codes_after.iter().any(|c| c.feat() == 0x0E);
    assert!(no_longer_promoted, "移除后不应再晋升");
}
