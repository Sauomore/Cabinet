//! HSH 20-bit 编码定义
//!
//! 层次语义哈希（Hierarchical Semantic Hashing）:
//! - feat: 4-bit 特征码（词性类别）
//! - sim:  8-bit 相似码（语义簇 ID）
//! - abs:  8-bit 绝对码（簇内完美哈希）
//!
//! 总空间：16 × 256 × 256 = 1,048,576 个唯一编码

/// HSH 20-bit 编码，内部用 u32 存储（高 12-bit 保留）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HSHCode(u32);

impl HSHCode {
    pub const BITS: u32 = 20;
    pub const MAX: u32 = 1 << 20; // 1,048,576

    /// 创建新的 HSH 编码
    /// # Panics
    /// 当 feat > 0xF, sim > 0xFF, 或 abs > 0xFF 时 panic
    pub fn new(feat: u8, sim: u8, abs: u8) -> Self {
        assert!(feat <= 0x0F, "feat 超出 4-bit 范围");
        assert!(sim <= 0xFF, "sim 超出 8-bit 范围");
        assert!(abs <= 0xFF, "abs 超出 8-bit 范围");
        let value = ((feat as u32) << 16) | ((sim as u32) << 8) | (abs as u32);
        HSHCode(value)
    }

    /// 从原始 u32 值创建（低 20-bit 有效）
    pub fn from_raw(value: u32) -> Self {
        HSHCode(value & ((1 << 20) - 1))
    }

    /// 获取原始 u32 值
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// 获取特征码（高 4-bit）
    pub fn feat(&self) -> u8 {
        ((self.0 >> 16) & 0x0F) as u8
    }

    /// 获取相似码（中 8-bit）
    pub fn sim(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// 获取绝对码（低 8-bit）
    pub fn abs(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// 打包为 3 bytes（大端序，高 4-bit 补零）
    /// 格式：[0000_feat][sim][abs]
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            ((self.0 >> 16) & 0x0F) as u8, // 0000 + feat
            ((self.0 >> 8) & 0xFF) as u8,   // sim
            (self.0 & 0xFF) as u8,          // abs
        ]
    }

    /// 从 3 bytes 解码
    pub fn from_bytes(b: [u8; 3]) -> Self {
        let feat = (b[0] & 0x0F) as u32;
        let sim = b[1] as u32;
        let abs = b[2] as u32;
        HSHCode((feat << 16) | (sim << 8) | abs)
    }

    /// 获取 match level 对应的 u16 前缀键
    /// 用于索引层前缀扫描：高 8-bit sim + 低 8-bit abs = u16
    pub fn drawer_key(&self) -> u16 {
        ((self.sim() as u16) << 8) | (self.abs() as u16)
    }

    /// 获取仅含 feat 和 sim 的前缀（用于同簇匹配）
    pub fn sim_prefix(&self) -> u16 {
        (self.sim() as u16) << 8
    }

    /// 编码为 u32 整数（便于 Python 层传递）
    pub fn to_u32(&self) -> u32 {
        self.0
    }

    /// 从 u32 整数解码
    pub fn from_u32(value: u32) -> Self {
        Self::from_raw(value)
    }

    /// 编码 HSH 序列为字节流（含长度前缀）
    pub fn encode_seq(codes: &[HSHCode]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + codes.len() * 3);
        buf.extend_from_slice(&(codes.len() as u32).to_be_bytes());
        for code in codes {
            buf.extend_from_slice(&code.to_bytes());
        }
        buf
    }

    /// 从字节流解码 HSH 序列（含长度前缀）
    pub fn decode_seq(buf: &[u8]) -> Option<Vec<HSHCode>> {
        if buf.len() < 4 {
            return None;
        }
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if buf.len() < 4 + len * 3 {
            return None;
        }
        let mut codes = Vec::with_capacity(len);
        for i in 0..len {
            let offset = 4 + i * 3;
            codes.push(HSHCode::from_bytes([buf[offset], buf[offset + 1], buf[offset + 2]]));
        }
        Some(codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsh_roundtrip() {
        let code = HSHCode::new(0x0A, 0x42, 0xF0);
        assert_eq!(code.feat(), 0x0A);
        assert_eq!(code.sim(), 0x42);
        assert_eq!(code.abs(), 0xF0);
        assert_eq!(code.raw(), (0x0Au32 << 16) | (0x42u32 << 8) | 0xF0u32);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let code = HSHCode::new(0x0F, 0xFF, 0xFF);
        let bytes = code.to_bytes();
        assert_eq!(bytes, [0x0F, 0xFF, 0xFF]);
        let decoded = HSHCode::from_bytes(bytes);
        assert_eq!(code, decoded);
    }

    #[test]
    fn test_from_raw() {
        let code = HSHCode::from_raw(0x00AB_CDEF);
        // 只保留低 20-bit: 0xB_CDEF
        assert_eq!(code.feat(), 0x0B);
        assert_eq!(code.sim(), 0xCD);
        assert_eq!(code.abs(), 0xEF);
    }

    #[test]
    fn test_seq_codec() {
        let codes = vec![
            HSHCode::new(0x0, 0x1, 0x2),
            HSHCode::new(0xF, 0xFF, 0xFF),
            HSHCode::new(0x5, 0x42, 0x88),
        ];
        let encoded = HSHCode::encode_seq(&codes);
        let decoded = HSHCode::decode_seq(&encoded).unwrap();
        assert_eq!(codes, decoded);
    }

    #[test]
    fn test_max_value() {
        let max = HSHCode::new(0x0F, 0xFF, 0xFF);
        assert_eq!(max.raw(), HSHCode::MAX - 1);
    }
}
