use std::time::Instant;

fn main() {
    let encoder = cabinet_hsh::Encoder::new();
    let text = "用户明天下午3点开会，准备PPT。".repeat(100);
    
    let start = Instant::now();
    let codes = encoder.encode(&text).unwrap();
    let elapsed = start.elapsed();
    
    let words = text.chars().filter(|&c| c == '，' || c == '。').count() + 1;
    println!("编码 {} 词: {:?}", words, elapsed);
    println!("HSH 序列: {:?}", codes.iter().map(|c| c.raw()).collect::<Vec<_>>());
}
