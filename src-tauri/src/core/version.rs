//! 协议版本枚举。
//!
//! D2 存档版本历史：
//! - v96/v97: 经典 LoD 老版
//! - v98: D2R LoD 首发
//! - v105: D2R（当前 D2RMM 主要目标）
//! - v111: D2R LoD 2.6+

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolVersion {
    V96 = 96,
    V97 = 97,
    V98 = 98,
    V105 = 105,
    V111 = 111,
}

impl ProtocolVersion {
    /// 从 raw u32 解析版本（用于 header 读取）。
    pub fn from_u32(v: u32) -> Option<Self> {
        Some(match v {
            96 => Self::V96,
            97 => Self::V97,
            98 => Self::V98,
            105 => Self::V105,
            111 => Self::V111,
            _ => return None,
        })
    }

    /// 当前解析使用的主要版本（D2R v105 是 D2RMM 模组主目标）。
    pub const CURRENT: Self = Self::V105;

    /// 是否包含 Page 3b 字段（v97+ 才有）。
    pub fn has_page_field(self) -> bool {
        self as u32 >= 97
    }

    /// version_bits 宽度：D2R = 3b，老版 = 10b。
    pub fn version_bits(self) -> u8 {
        if (self as u32) >= 105 { 3 } else { 10 }
    }

    /// socket_count_bits：D2R = 3b，老版 = 1b。
    pub fn socket_count_bits(self) -> u8 {
        if (self as u32) >= 105 { 3 } else { 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u32() {
        assert_eq!(ProtocolVersion::from_u32(96), Some(ProtocolVersion::V96));
        assert_eq!(ProtocolVersion::from_u32(105), Some(ProtocolVersion::V105));
        assert_eq!(ProtocolVersion::from_u32(111), Some(ProtocolVersion::V111));
        assert_eq!(ProtocolVersion::from_u32(0), None);
        assert_eq!(ProtocolVersion::from_u32(200), None);
    }

    #[test]
    fn test_has_page_field() {
        assert!(!ProtocolVersion::V96.has_page_field());
        assert!(ProtocolVersion::V97.has_page_field());
        assert!(ProtocolVersion::V105.has_page_field());
        assert!(ProtocolVersion::V111.has_page_field());
    }

    #[test]
    fn test_version_bits() {
        assert_eq!(ProtocolVersion::V96.version_bits(), 10);
        assert_eq!(ProtocolVersion::V105.version_bits(), 3);
        assert_eq!(ProtocolVersion::V111.version_bits(), 3);
    }

    #[test]
    fn test_socket_count_bits() {
        assert_eq!(ProtocolVersion::V96.socket_count_bits(), 1);
        assert_eq!(ProtocolVersion::V105.socket_count_bits(), 3);
    }

    #[test]
    fn test_current() {
        assert_eq!(ProtocolVersion::CURRENT, ProtocolVersion::V105);
    }
}