//! VByte / Delta 压缩编解码
//!
//! VByte：用最高位作为延续标志，其余 7-bit 存储数值
//! Delta：对有序序列存储相邻差值（首个存绝对值）

/// 将 u64 编码为 VByte 字节序列
pub fn encode_vbyte(mut value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            buf.push(byte);
            break;
        } else {
            buf.push(byte | 0x80);
        }
    }
    buf
}

/// 从字节流解码 VByte（返回 (value, bytes_consumed)）
pub fn decode_vbyte(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if i > 9 {
            // u64 最多需要 10 字节 VByte
            return None;
        }
        let chunk = (byte & 0x7F) as u64;
        value |= chunk << shift;
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
    }
    None
}

/// Delta 编码：对有序 u64 序列编码
/// 首个存绝对值，后续存与前一个的差值
pub fn encode_delta(values: &[u64]) -> Vec<u8> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut buf = Vec::new();
    let mut prev = 0u64;
    for (i, &v) in values.iter().enumerate() {
        let delta = if i == 0 { v } else { v - prev };
        buf.extend_from_slice(&encode_vbyte(delta));
        prev = v;
    }
    buf
}

/// Delta 解码：从字节流恢复有序 u64 序列
/// 需提供元素数量（因为 VByte 变长，无法从边界推断）
pub fn decode_delta(bytes: &[u8], count: usize) -> Option<Vec<u64>> {
    let mut values = Vec::with_capacity(count);
    let mut offset = 0usize;
    let mut prev = 0u64;
    for i in 0..count {
        if offset >= bytes.len() {
            return None;
        }
        let (delta, consumed) = decode_vbyte(&bytes[offset..])?;
        let value = if i == 0 { delta } else { prev + delta };
        values.push(value);
        prev = value;
        offset += consumed;
    }
    Some(values)
}

/// 对 u32 序列的 Delta 编码（位置信息常用 u32）
pub fn encode_delta_u32(values: &[u32]) -> Vec<u8> {
    encode_delta(&values.iter().map(|&v| v as u64).collect::<Vec<_>>())
}

pub fn decode_delta_u32(bytes: &[u8], count: usize) -> Option<Vec<u32>> {
    decode_delta(bytes, count).map(|v| v.into_iter().map(|x| x as u32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vbyte_roundtrip() {
        let tests = [0u64, 1, 127, 128, 255, 256, 16383, 16384, u64::MAX / 2];
        for &v in &tests {
            let enc = encode_vbyte(v);
            let (dec, consumed) = decode_vbyte(&enc).unwrap();
            assert_eq!(dec, v, "v={}", v);
            assert_eq!(consumed, enc.len());
        }
    }

    #[test]
    fn test_delta_roundtrip() {
        let values = vec![100u64, 105, 110, 200, 201, 300];
        let enc = encode_delta(&values);
        let dec = decode_delta(&enc, values.len()).unwrap();
        assert_eq!(dec, values);
    }

    #[test]
    fn test_delta_u32_roundtrip() {
        let values = vec![10u32, 15, 20, 100, 101];
        let enc = encode_delta_u32(&values);
        let dec = decode_delta_u32(&enc, values.len()).unwrap();
        assert_eq!(dec, values);
    }
}
