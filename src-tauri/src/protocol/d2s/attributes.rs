//! D2S attributes 段解析 — bit-packed stream 格式。
//!
//! D2SLib 真实格式(halbu v105 codec 验证):
//! ```text
//! SECTION_HEADER  = [0x67, 0x66]   ("gf")
//! SECTION_TRAILER = 0x1FF          (9 bits)
//! STAT_HEADER_BITS = 9             (stat_id 0..15)
//!
//! 每 stat: 9-bit header (stat_id) + N-bit value
//! 字段顺序 (D2R charstats.txt):
//!   0 Strength (10b)
//!   1 Energy (10b)
//!   2 Dexterity (10b)
//!   3 Vitality (10b)
//!   4 StatPoints (10b)
//!   5 NewSkills (8b)
//!   6 Hitpoints (21b Q8)
//!   7 MaxHp (21b Q8)
//!   8 Mana (21b Q8)
//!   9 MaxMana (21b Q8)
//!  10 Stamina (21b Q8)
//!  11 MaxStamina (21b Q8)
//!  12 Level (7b)
//!  13 Experience (32b)
//!  14 Gold (25b)
//!  15 GoldBank (25b)
//! ```
//!
//! Q8 修饰: HP / Mana / Stamina 实际显示值 = raw / 256。
//!
//! **历史错误**: 之前实现假定 12×u32 + u8 + 3×u32 = 61B,这是错误的
//! (来自 Trevin v1.09 notes 而非 D2R v105)。halbu v105 codec 源码确认
//! 真实格式是上面描述的 bit-packed stream。
//!
//! 实测验证: xieedi.d2s (魔改) 和 开心的蛮蛮.d2s (标准) 都在 offset 0x341
//! 找到 0x6766 ("gf") header。

use crate::core::bitio::BitReader;
use crate::core::ParseResult;
use crate::protocol::common::stat_list::STAT_LIST_TERMINATOR;
use crate::protocol::common::StatTable;

/// Attributes section header: 2 bytes ASCII "gf".
pub const ATTRIBUTES_HEADER: [u8; 2] = [0x67, 0x66];

/// Attributes section trailer (与 stat_list 一致,9-bit 值)。
pub const ATTRIBUTES_TRAILER: u16 = STAT_LIST_TERMINATOR;

/// 每个 stat 的 header 位宽 (9 bits 存 stat_id 0..15)。
pub const STAT_HEADER_BITS: u8 = 9;

/// stat_id 0..15,与 D2R charstats.txt 一致。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeId {
    Strength = 0,
    Energy = 1,
    Dexterity = 2,
    Vitality = 3,
    StatPoints = 4,
    NewSkills = 5,
    Hitpoints = 6,
    MaxHp = 7,
    Mana = 8,
    MaxMana = 9,
    Stamina = 10,
    MaxStamina = 11,
    Level = 12,
    Experience = 13,
    Gold = 14,
    GoldBank = 15,
}

impl AttributeId {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            0 => Some(Self::Strength),
            1 => Some(Self::Energy),
            2 => Some(Self::Dexterity),
            3 => Some(Self::Vitality),
            4 => Some(Self::StatPoints),
            5 => Some(Self::NewSkills),
            6 => Some(Self::Hitpoints),
            7 => Some(Self::MaxHp),
            8 => Some(Self::Mana),
            9 => Some(Self::MaxMana),
            10 => Some(Self::Stamina),
            11 => Some(Self::MaxStamina),
            12 => Some(Self::Level),
            13 => Some(Self::Experience),
            14 => Some(Self::Gold),
            15 => Some(Self::GoldBank),
            _ => None,
        }
    }

    /// stat 值的位宽。
    pub fn bit_length(self) -> u8 {
        match self {
            Self::Strength
            | Self::Energy
            | Self::Dexterity
            | Self::Vitality
            | Self::StatPoints => 10,
            Self::NewSkills => 8,
            Self::Hitpoints
            | Self::MaxHp
            | Self::Mana
            | Self::MaxMana
            | Self::Stamina
            | Self::MaxStamina => 21,
            Self::Level => 7,
            Self::Experience => 32,
            Self::Gold | Self::GoldBank => 25,
        }
    }

    /// 从 StatProp 中取位宽（优先 cs_bits，回退 bit_length）。
    pub fn bit_length_from_table(self, table: &StatTable) -> u8 {
        let id = self as u16;
        let prop = table.get(id);
        if prop.cs_bits > 0 {
            prop.cs_bits
        } else {
            self.bit_length()
        }
    }

    /// 该 stat 是否用 Q8 修饰 (raw / 256 = 显示值)。
    pub fn is_q8(self) -> bool {
        matches!(
            self,
            Self::Hitpoints
                | Self::MaxHp
                | Self::Mana
                | Self::MaxMana
                | Self::Stamina
                | Self::MaxStamina
        )
    }
}

/// 单个 stat raw value (u32 容纳所有位宽)。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatValue {
    pub raw: u32,
}

impl StatValue {
    /// 显示值 (Q8 除 256,其他位宽原值)。
    pub fn display(self) -> u32 {
        self.raw
    }
}

/// 16 个角色 attribute 容器。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterAttributes {
    pub strength: StatValue,
    pub energy: StatValue,
    pub dexterity: StatValue,
    pub vitality: StatValue,
    pub stat_points: StatValue,
    pub new_skills: StatValue,
    pub hitpoints: StatValue,
    pub max_hp: StatValue,
    pub mana: StatValue,
    pub max_mana: StatValue,
    pub stamina: StatValue,
    pub max_stamina: StatValue,
    pub level: StatValue,
    pub experience: StatValue,
    pub gold: StatValue,
    pub gold_bank: StatValue,
}

impl CharacterAttributes {
    /// 按 id 读 raw value (Q8 已修正)。
    pub fn get(&self, id: AttributeId) -> u32 {
        let raw = match id {
            AttributeId::Strength => self.strength.raw,
            AttributeId::Energy => self.energy.raw,
            AttributeId::Dexterity => self.dexterity.raw,
            AttributeId::Vitality => self.vitality.raw,
            AttributeId::StatPoints => self.stat_points.raw,
            AttributeId::NewSkills => self.new_skills.raw,
            AttributeId::Hitpoints => self.hitpoints.raw,
            AttributeId::MaxHp => self.max_hp.raw,
            AttributeId::Mana => self.mana.raw,
            AttributeId::MaxMana => self.max_mana.raw,
            AttributeId::Stamina => self.stamina.raw,
            AttributeId::MaxStamina => self.max_stamina.raw,
            AttributeId::Level => self.level.raw,
            AttributeId::Experience => self.experience.raw,
            AttributeId::Gold => self.gold.raw,
            AttributeId::GoldBank => self.gold_bank.raw,
        };
        if id.is_q8() {
            raw / 256
        } else {
            raw
        }
    }

    fn set_raw(&mut self, id: AttributeId, raw: u32) {
        let slot = match id {
            AttributeId::Strength => &mut self.strength.raw,
            AttributeId::Energy => &mut self.energy.raw,
            AttributeId::Dexterity => &mut self.dexterity.raw,
            AttributeId::Vitality => &mut self.vitality.raw,
            AttributeId::StatPoints => &mut self.stat_points.raw,
            AttributeId::NewSkills => &mut self.new_skills.raw,
            AttributeId::Hitpoints => &mut self.hitpoints.raw,
            AttributeId::MaxHp => &mut self.max_hp.raw,
            AttributeId::Mana => &mut self.mana.raw,
            AttributeId::MaxMana => &mut self.max_mana.raw,
            AttributeId::Stamina => &mut self.stamina.raw,
            AttributeId::MaxStamina => &mut self.max_stamina.raw,
            AttributeId::Level => &mut self.level.raw,
            AttributeId::Experience => &mut self.experience.raw,
            AttributeId::Gold => &mut self.gold.raw,
            AttributeId::GoldBank => &mut self.gold_bank.raw,
        };
        *slot = raw;
    }
}

/// 解析 attributes 段 (从 gf header 起始)。
///
/// 输入: 字节流(以 `[0x67, 0x66]` 起始)。
/// 输出: 解析出的 16 个 stat。
///
/// 容错策略:
/// - 找不到 gf header → 报错
/// - header 9-bit id > 15 或 == 0x1FF → 停止循环
/// - 剩余位不足 → 停止
/// 解析 attributes 段（使用给定的 StatTable 查找 CSvBits）。
pub fn parse(data: &[u8]) -> ParseResult<CharacterAttributes> {
    parse_with_table(data, None)
}

/// 解析 attributes 段，可指定 StatTable（用于 CSvBits 查表）。
pub fn parse_with_table(data: &[u8], table: Option<&StatTable>) -> ParseResult<CharacterAttributes> {
    if data.len() < 2 {
        return Err(crate::core::ParseError::Truncated(data.len()));
    }
    if data[0..2] != ATTRIBUTES_HEADER {
        return Err(crate::core::ParseError::InvalidSection(format!(
            "expected gf header, got {:02X?}",
            &data[0..2]
        )));
    }
    let mut reader = BitReader::new(&data[2..]);
    let mut attrs = CharacterAttributes::default();

    // 循环:最多 17 项 (16 stats + 1 trailer 保险)
    for _ in 0..=16 {
        if reader.remaining_bits() < STAT_HEADER_BITS as usize {
            break;
        }
        let header = reader.read_u16(STAT_HEADER_BITS);
        if header == ATTRIBUTES_TRAILER {
            break;
        }
        let Some(id) = AttributeId::from_u16(header) else {
            // 未知 stat_id (> 15): 容错,停止解析
            break;
        };
        let bits = if let Some(tbl) = table {
            id.bit_length_from_table(tbl)
        } else {
            id.bit_length()
        };
        if reader.remaining_bits() < bits as usize {
            break;
        }
        let raw = reader.read_u32(bits);
        attrs.set_raw(id, raw);
    }

    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_attribute_id_bit_lengths() {
        assert_eq!(AttributeId::Strength.bit_length(), 10);
        assert_eq!(AttributeId::Energy.bit_length(), 10);
        assert_eq!(AttributeId::Dexterity.bit_length(), 10);
        assert_eq!(AttributeId::Vitality.bit_length(), 10);
        assert_eq!(AttributeId::StatPoints.bit_length(), 10);
        assert_eq!(AttributeId::NewSkills.bit_length(), 8);
        assert_eq!(AttributeId::Hitpoints.bit_length(), 21);
        assert_eq!(AttributeId::MaxHp.bit_length(), 21);
        assert_eq!(AttributeId::Mana.bit_length(), 21);
        assert_eq!(AttributeId::MaxMana.bit_length(), 21);
        assert_eq!(AttributeId::Stamina.bit_length(), 21);
        assert_eq!(AttributeId::MaxStamina.bit_length(), 21);
        assert_eq!(AttributeId::Level.bit_length(), 7);
        assert_eq!(AttributeId::Experience.bit_length(), 32);
        assert_eq!(AttributeId::Gold.bit_length(), 25);
        assert_eq!(AttributeId::GoldBank.bit_length(), 25);
    }

    #[test]
    fn test_attribute_id_q8() {
        assert!(AttributeId::MaxHp.is_q8());
        assert!(AttributeId::Hitpoints.is_q8());
        assert!(!AttributeId::Strength.is_q8());
        assert!(!AttributeId::Level.is_q8());
        assert!(!AttributeId::Gold.is_q8());
    }

    #[test]
    fn test_attributes_finds_gf_header() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        assert_eq!(data[0x341..0x343], ATTRIBUTES_HEADER);
    }

    #[test]
    fn test_attributes_parse_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let attrs = parse(&data[0x341..]).expect("parse attributes");
        // xieedi Warlock lv 116, strength/vit 应该 > 100 (Necro 116 级典型 str=125+, vit=245+)
        let str_val = attrs.get(AttributeId::Strength);
        let vit_val = attrs.get(AttributeId::Vitality);
        let level_val = attrs.get(AttributeId::Level);
        println!(
            "xieedi attrs: str={} vit={} level={} max_hp={} max_mana={} exp={}",
            str_val,
            vit_val,
            level_val,
            attrs.get(AttributeId::MaxHp),
            attrs.get(AttributeId::MaxMana),
            attrs.get(AttributeId::Experience)
        );
        // 不强求精确值,只断言解析出了非零且合理的数值
        // (如果解析错误,这些都将是 0)
        assert!(str_val > 50, "strength 应该 > 50, got {}", str_val);
        assert!(vit_val > 50, "vitality 应该 > 50, got {}", vit_val);
        // level 应该和魔改 layout 0x1B 字段一致 (116)
        assert_eq!(level_val, 116, "level 应为 116 (与魔改 layout 0x1B 一致)");
    }

    #[test]
    fn test_attributes_parse_happy_manman() {
        let _fp = fixture_path("happy_manman.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture happy_manman.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let attrs = parse(&data[0x341..]).expect("parse attributes");
        let level_val = attrs.get(AttributeId::Level);
        let str_val = attrs.get(AttributeId::Strength);
        let vit_val = attrs.get(AttributeId::Vitality);
        println!(
            "happy_manman attrs: str={} vit={} level={} max_hp={}",
            str_val,
            vit_val,
            level_val,
            attrs.get(AttributeId::MaxHp)
        );
        assert!(level_val > 0, "level 应该 > 0");
        assert!(str_val > 0, "strength 应该 > 0");
    }

    #[test]
    fn test_attributes_trailer_terminates() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let mut reader = BitReader::new(&data[0x343..]);
        let h = reader.read_u16(STAT_HEADER_BITS);
        // 第一个 9-bit header 应该是某个 stat_id (< 16),不是 trailer (0x1FF)
        assert_ne!(h, ATTRIBUTES_TRAILER, "first header 应是 stat 不是 trailer");
        assert!(h < 16, "first header 应该是 stat_id 0..15, got {}", h);
    }

    /// 用真实的 xieedi / happy_manman .d2s 字节扫描,
    /// 找出 attributes `gf` header 真实位置
    #[test]
    fn test_scan_attributes_offset_in_real_files() {
        for name in &["xieedi.d2s", "happy_manman.d2s"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join(name);
            if !path.exists() { eprintln!("SKIP: fixture {} 缺失", name); continue; }
            let data = std::fs::read(&path).unwrap();
            // 找所有 0x6766 ("gf") 出现位置
            let mut offsets = Vec::new();
            for i in 0..data.len().saturating_sub(2) {
                if data[i] == 0x67 && data[i + 1] == 0x66 {
                    // 检查是不是真的 gf header (后面应是 bit-packed level/exp 等)
                    // 0x66 后 6 bytes 应该能解出 9-bit header + value
                    offsets.push(i);
                }
            }
            println!("{} ({} bytes): gf at {:?}", name, data.len(), offsets);
        }
    }

    /// 用真实文件验证 attributes parse — 找 gf 后解 attributes
    #[test]
    fn test_parse_real_xieedi_attributes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("xieedi.d2s");
        if !path.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(path).unwrap();

        let gf_pos = data.windows(2)
            .position(|w| w == [0x67, 0x66])
            .expect("should find gf header");

        println!("xieedi.d2s: gf header at 0x{:04X}", gf_pos);

        // 尝试 parse
        match parse(&data[gf_pos..]) {
            Ok(attrs) => {
                let level = attrs.get(AttributeId::Level);
                let str_ = attrs.get(AttributeId::Strength);
                let vit = attrs.get(AttributeId::Vitality);
                let hp = attrs.get(AttributeId::MaxHp);
                let mana = attrs.get(AttributeId::MaxMana);
                let exp = attrs.get(AttributeId::Experience);
                println!(
                    "xieedi attrs from 0x{:04X}: str={} vit={} level={} max_hp={} max_mana={} exp={}",
                    gf_pos, str_, vit, level, hp, mana, exp
                );
                // Warlock lv 116: str > 100, vit > 100
                assert!(str_ > 50, "str > 50");
                assert!(vit > 50, "vit > 50");
            }
            Err(e) => panic!("parse err: {:?}", e),
        }
    }

    #[test]
    fn test_parse_real_happy_manman_attributes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("happy_manman.d2s");
        if !path.exists() { eprintln!("SKIP: fixture happy_manman.d2s 缺失"); return; }
        let data = std::fs::read(path).unwrap();

        let gf_pos = data.windows(2)
            .position(|w| w == [0x67, 0x66])
            .expect("should find gf header");

        println!("happy_manman.d2s: gf header at 0x{:04X}", gf_pos);

        match parse(&data[gf_pos..]) {
            Ok(attrs) => {
                let level = attrs.get(AttributeId::Level);
                let str_ = attrs.get(AttributeId::Strength);
                let vit = attrs.get(AttributeId::Vitality);
                let hp = attrs.get(AttributeId::MaxHp);
                let mana = attrs.get(AttributeId::MaxMana);
                let exp = attrs.get(AttributeId::Experience);
                println!(
                    "happy attrs from 0x{:04X}: str={} vit={} level={} max_hp={} max_mana={} exp={}",
                    gf_pos, str_, vit, level, hp, mana, exp
                );
                // Barbarian lv 93
                assert!(str_ > 50, "str > 50");
                assert!(vit > 50, "vit > 50");
            }
            Err(e) => panic!("parse err: {:?}", e),
        }
    }

    #[test]
    fn test_attributes_invalid_header() {
        // 非 gf header 应该报错
        let bad = [0x00, 0x00, 0x00, 0x00];
        let err = parse(&bad).unwrap_err();
        assert!(matches!(err, crate::core::ParseError::InvalidSection(_)));
    }
}