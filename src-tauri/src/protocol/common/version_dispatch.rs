//! 协议版本分发表（★ 核心抽象 ★）
//!
//! D2SLib 在不同版本下字段宽度不同。本模块集中描述差异，
//! 所有 `Item::read_*` 子函数按 `ProtocolVersion` 查表决定位宽。

use crate::core::ProtocolVersion;

/// 字段位宽差异表（Item 头部 compact + complete）。
///
/// 由 D2SLib `Items.cs` 推断：
/// - v96/v97 老版：socket_count=1b，version_bits=10b，无 spell offset (chronicle)
/// - v98/v105 D2R：socket_count=3b，version_bits=3b，v105+ 有 chronicle (52b)
#[derive(Debug, Clone, Copy)]
pub struct FieldSet;

impl FieldSet {
    /// Compact header 中的 socket 计数位宽
    #[inline]
    pub fn socket_count_bits(v: ProtocolVersion) -> u8 {
        v.socket_count_bits()
    }

    /// Compact header 中的 version 位宽
    #[inline]
    pub fn version_bits(v: ProtocolVersion) -> u8 {
        v.version_bits()
    }

    /// v105+ 才有的 chronicle（16b monsterId + 32b timestamp + 4b padding = 52b）
    #[inline]
    pub fn has_chronicle(v: ProtocolVersion) -> bool {
        (v as u32) >= 105
    }

    /// 是否读 ItemPage 3b 字段（v97+ 才有）
    #[inline]
    pub fn has_page_field(v: ProtocolVersion) -> bool {
        v.has_page_field()
    }

    /// Complete header 中的个性化名（personalized_name）位宽
    /// 仅 `personalized` flag 为 1 时存在
    #[inline]
    pub fn has_personalized_name() -> bool {
        true
    }

    /// realm data flag 是否存在（v100+）
    #[inline]
    pub fn has_realm_data(v: ProtocolVersion) -> bool {
        (v as u32) >= 100
    }

    /// Sub-property 自动展开（itemstatcost.txt 中 Save_Param_Bits 控制）：
    /// - sid 17 (item_maxdamage_percent) 自动跟 18
    /// - sid 52 (magicmindam) 自动跟 53
    /// - sid 54 (coldmindam) 自动跟 55, 56
    /// - sid 57 (poisonmindam) 自动跟 58, 59
    pub fn sub_property_count(stat_id: u16) -> u8 {
        match stat_id {
            17 => 1,  // +18 (item_maxdamage)
            50 => 1,  // +51 (lightmaxdam)
            52 => 1,  // +53 (magicmaxdam)
            54 => 2,  // +55, +56 (coldmindam, coldmaxdam)
            57 => 2,  // +58, +59 (poisonmindam, poisonmaxdam)
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v96_no_page_no_chronicle() {
        assert!(!FieldSet::has_page_field(ProtocolVersion::V96));
        assert!(!FieldSet::has_chronicle(ProtocolVersion::V96));
        assert!(!FieldSet::has_realm_data(ProtocolVersion::V96));
    }

    #[test]
    fn test_v105_has_chronicle_no_realm() {
        assert!(FieldSet::has_page_field(ProtocolVersion::V105));
        assert!(FieldSet::has_chronicle(ProtocolVersion::V105));
        assert!(FieldSet::has_realm_data(ProtocolVersion::V105));
    }

    #[test]
    fn test_socket_count_bits() {
        assert_eq!(FieldSet::socket_count_bits(ProtocolVersion::V96), 1);
        assert_eq!(FieldSet::socket_count_bits(ProtocolVersion::V105), 3);
    }

    #[test]
    fn test_sub_property_count() {
        assert_eq!(FieldSet::sub_property_count(17), 1);
        assert_eq!(FieldSet::sub_property_count(52), 1);
        assert_eq!(FieldSet::sub_property_count(54), 2);
        assert_eq!(FieldSet::sub_property_count(57), 2);
        assert_eq!(FieldSet::sub_property_count(0), 0);
        assert_eq!(FieldSet::sub_property_count(99), 0);
    }
}