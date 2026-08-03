//! D2S 角色存档 header。
//!
//! D2R v105 标准 layout（construct_adapter/d2s.py RawCharacterData 验证）:
//! ```text
//! 0x00: magic       0xAA55AA55 (4B)         ← D2R 与 d2i 共用
//! 0x04: header_size 0x69 = 105 (u32 LE)     ← D2R 扩展头部总长
//! 0x08: filesize    (u32 LE)
//! 0x0C: checksum    (u32 LE)                ← D2 滚动校验和
//! 0x10: active_weapon (u32 LE)              ← 0=主, 1=武器切换
//! 0x14: menu_layout  (u32 LE)               ← 仓库样式/菜单布局
//! 0x18: class_id     (u8)                   ← 0=Amazon..7=Warlock
//! 0x19: status_bits  (u8)                   ← bit2=HC, bit3=Dead, bit4=Expansion
//! 0x1A: num_skills   (u8)                   ← 已分配技能点数
//! 0x1B: level        (u8)
//! 0x1C: reserved     (4B) 全零
//! 0x20: save_timestamp (i32 LE)             ← Unix 时间戳
//! 0x24: unused       (4B) 0xFFFFFFFF
//! 0x28: hotkeys      10 × u32 LE (40B)
//! 0x50: left_mouse   (u32 LE)               ← 左键技能 ID
//! 0x54: right_mouse  5 × u32 LE (20B)       ← 右键技能(含换手)
//! 0x68: end_marker   (u8) = 0x00
//! ────────────────────────────────────
//! 总计: 105 bytes (0x69)
//! ```
//!
//! 角色名在 compat 段 (0x69..0x193) 偏移 0xC8 处 UTF-8 编码。

use crate::core::{ParseError, ParseResult, ProtocolVersion};

/// D2S 文件 magic（D2R v99/v105） = `0xAA55AA55`
pub const D2S_MAGIC: &[u8; 4] = &[0x55, 0xAA, 0x55, 0xAA];

/// D2S header 105B 结构（标准 D2R v105 layout）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct D2SHeader {
    /// 0x04 — D2R 中实际是 header_size=0x69，保留字段名兼容 ProtocolVersion 映射
    pub version_raw: u32,
    pub filesize: u32,
    pub checksum: u32,
    pub active_weapon: u32,
    /// UTF-8 角色名（来自 compat 段偏移 0xC8）
    pub name: String,
    /// 0x19 — bit2=HC, bit3=Dead, bit4=Expansion
    pub status_flags: u8,
    /// 0x18 — 职业 ID
    pub class: u8,
    /// 0x1A — 已分配技能点数
    pub num_skills: u8,
    /// 0x14 — 菜单布局/仓库样式
    pub menu_layout: u32,
    /// 0x20 — Unix 时间戳
    pub save_timestamp: u32,
    /// 0xA8..0xAB — 三难度 Act 位置（byte[0]=Normal, [1]=NM, [2]=Hell）
    /// bit7 = active (是否进入过), bits 0-2 = act index (0..4)
    pub location: [u8; 3],
    /// 0x28 — 10 个技能快捷键
    pub hotkeys: [u32; 10],
    /// 0x50 — 左键技能 ID
    pub left_mouse_skill: u32,
    /// 0x54 — 右键技能(5 个含换手)
    pub right_mouse_skills: [u32; 5],
    /// 0x68 — 结束标记
    pub end_marker: u8,
}

impl D2SHeader {
    /// 标准 D2R v105 105B header 解析。
    ///
    /// 字段布局见模块文档
    /// (construct_adapter/d2s.py RawCharacterData 验证)。
    pub fn from_bytes(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 0x69 {
            return Err(ParseError::Truncated(data.len()));
        }
        if &data[0..4] != D2S_MAGIC {
            return Err(ParseError::D2SMagic([data[0], data[1], data[2]]));
        }
        let version_raw = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let filesize = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let checksum = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let active_weapon = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let menu_layout = u32::from_le_bytes([data[0x14], data[0x15], data[0x16], data[0x17]]);
        let class = data[0x18];
        let status_flags = data[0x19];
        let num_skills = data[0x1A];
        let save_timestamp = u32::from_le_bytes([data[0x20], data[0x21], data[0x22], data[0x23]]);

        let mut hotkeys = [0u32; 10];
        for i in 0..10 {
            let off = 0x28 + i * 4;
            hotkeys[i] = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        }
        let left_mouse_skill = u32::from_le_bytes([data[0x50], data[0x51], data[0x52], data[0x53]]);
        let mut right_mouse_skills = [0u32; 5];
        for i in 0..5 {
            let off = 0x54 + i * 4;
            right_mouse_skills[i] = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        }
        let end_marker = data[0x68];
        let location = [data[0xA8], data[0xA9], data[0xAA]];

        // name: 多种位置尝试 (优先取更长名)
        let name = if data.len() >= 0x193 {
            let compat = &data[0x69..0x193];
            // 1) file+0x12B (d2emu mod 扩展名称偏移)
            if let Some(n) = read_utf8_name(data, 0x12B) { n }
            // 2) compat+0xC8  (标准 v105 layout)
            else if let Some(n) = read_utf8_name(compat, 0xC8) { n }
            // 3) file+0x12C (d2emu 兼容)
            else if let Some(n) = read_utf8_name(data, 0x12C) { n }
            // 4) 宽扫描: 在 compat 段中找可打印 ASCII/CJK 名
            else { scan_name_in_compat(compat) }
        } else {
            String::new()
        };

        Ok(Self {
            version_raw,
            filesize,
            checksum,
            active_weapon,
            name,
            status_flags,
            class,
            num_skills,
            menu_layout,
            save_timestamp,
            location,
            hotkeys,
            left_mouse_skill,
            right_mouse_skills,
            end_marker,
        })
    }

    /// 协议版本（与 `ProtocolVersion` 枚举对应）。
    pub fn protocol_version(&self) -> ProtocolVersion {
        ProtocolVersion::from_u32(self.version_raw).unwrap_or(ProtocolVersion::V97)
    }

    /// 角色职业（Amazon/Sorceress/...）。
    pub fn character_class(&self) -> CharacterClass {
        CharacterClass::from(self.class)
    }

    /// 是否 hardcore。
    pub fn is_hardcore(&self) -> bool {
        self.status_flags & 0x04 != 0
    }

    /// 是否已死（专家模式）。
    pub fn is_dead(&self) -> bool {
        self.status_flags & 0x08 != 0
    }

    /// 是否扩展（LoD/D2R）。D2R 中所有角色均为资料片角色,此位始终为 1。
    /// Python construct_adapter 确认 bit4(0x10) 为 Expansion 标志。
    pub fn is_expansion(&self) -> bool {
        self.status_flags & 0x10 != 0
    }

    /// 兼容性: last_played 映射到 save_timestamp。
    pub fn last_played(&self) -> u32 {
        self.save_timestamp
    }
}

/// 从指定偏移读 null-terminated UTF-8 字符串。
pub(crate) fn read_utf8_name(data: &[u8], offset: usize) -> Option<String> {
    if offset >= data.len() { return None; }
    let slice = &data[offset..];
    let end = slice.iter().position(|&b| b == 0)?;
    if end == 0 { return None; }
    // 至少 2 字符 (ASCII) 或 3 字节 (CJK)
    if end < 2 && (slice[0] < 0x80 || end < 3) { return None; }
    // 检查是否主要含可打印字符
    let printable = slice[..end].iter().filter(|&&b| (0x20..0x7F).contains(&b) || b >= 0x80).count();
    if printable < end.saturating_sub(1) { return None; }
    String::from_utf8_lossy(&slice[..end]).to_string().into()
}

/// 在 compat 段中扫描可能的角色名。
fn scan_name_in_compat(compat: &[u8]) -> String {
    // 扫描所有 null-terminated 字符串,取最长的合理中文/英文名
    let mut best = String::new();
    let mut i = 0;
    while i < compat.len() {
        // 跳过前导非 ASCII/CJK 字节
        while i < compat.len() && compat[i] != 0 && (compat[i] < 0x20 || compat[i] == 0x7F) {
            i += 1;
        }
        if i >= compat.len() { break; }
        let start = i;
        while i < compat.len() && compat[i] != 0 { i += 1; }
        let raw = &compat[start..i];
        if raw.len() >= 2
            && let Ok(s) = String::from_utf8(raw.to_vec()) {
                let clean: String = s.chars().filter(|&c| c.is_ascii_alphanumeric() || c >= '\u{4E00}').collect();
                if clean.len() >= 2 && clean.len() <= 16 && clean.len() > best.len() {
                    best = clean;
                }
            }
        i += 1;
    }
    best
}

/// 角色职业枚举（D2R 7 个基础 + 1 个 Warlock + 2 个 DLC = 10 个）。
///
/// D2SLib charstats.txt 顺序:
/// Amazon=0, Sorceress=1, Necromancer=2, Paladin=3, Barbarian=4, Druid=5,
/// Assassin=6, **Warlock=7** (D2R new class)。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Amazon = 0,
    Sorceress = 1,
    Necromancer = 2,
    Paladin = 3,
    Barbarian = 4,
    Druid = 5,
    Assassin = 6,
    Warlock = 7,
    Unknown(u8),
}

impl CharacterClass {
    pub fn from(v: u8) -> Self {
        match v {
            0 => Self::Amazon,
            1 => Self::Sorceress,
            2 => Self::Necromancer,
            3 => Self::Paladin,
            4 => Self::Barbarian,
            5 => Self::Druid,
            6 => Self::Assassin,
            7 => Self::Warlock,
            _ => Self::Unknown(v),
        }
    }

    pub fn name_en(self) -> &'static str {
        match self {
            Self::Amazon => "Amazon",
            Self::Sorceress => "Sorceress",
            Self::Necromancer => "Necromancer",
            Self::Paladin => "Paladin",
            Self::Barbarian => "Barbarian",
            Self::Druid => "Druid",
            Self::Assassin => "Assassin",
            Self::Warlock => "Warlock",
            Self::Unknown(_) => "Unknown",
        }
    }

    pub fn name_cn(self) -> &'static str {
        match self {
            Self::Amazon => "亚马逊",
            Self::Sorceress => "女巫",
            Self::Necromancer => "死灵法师",
            Self::Paladin => "圣骑士",
            Self::Barbarian => "野蛮人",
            Self::Druid => "德鲁伊",
            Self::Assassin => "刺客",
            Self::Warlock => "术士",
            Self::Unknown(_) => "未知",
        }
    }

    pub fn name_tw(self) -> &'static str {
        match self {
            Self::Amazon => "亞馬遜",
            Self::Sorceress => "女巫",
            Self::Necromancer => "死靈法師",
            Self::Paladin => "聖騎士",
            Self::Barbarian => "野蠻人",
            Self::Druid => "德魯伊",
            Self::Assassin => "刺客",
            Self::Warlock => "術士",
            Self::Unknown(_) => "未知",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_basic() {
        // 标准 D2R v105 layout:
        // 0x00: magic
        // 0x04: header_size=0x69
        // 0x08: filesize
        // 0x0C: checksum
        // 0x10: active_weapon
        // 0x14: menu_layout
        // 0x18: class = Sorceress (1)
        // 0x19: status = 0x14 (HC + Expansion)
        // 0x1A: num_skills = 0
        // 0x1B: level = 45
        // 0x131: name UTF-8 = "TestChar\0"
        let data = make_standard_d2s_header(1, 0x14, 45, "TestChar");
        let h = D2SHeader::from_bytes(&data).unwrap();
        assert_eq!(h.version_raw, 0x69, "header_size=0x69");
        assert_eq!(h.class, 1);
        assert_eq!(h.character_class(), CharacterClass::Sorceress);
        assert!(h.is_hardcore(), "0x14 bit2=HC");
        assert!(h.is_expansion(), "0x14 bit4=Expansion");
        assert!(!h.is_dead(), "0x14 bit3=0");
        assert_eq!(h.num_skills, 0);
        assert_eq!(h.save_timestamp, 0);
        assert_eq!(h.name, "TestChar");
    }

    #[test]
    fn test_parse_header_with_name() {
        // 标准 layout: name @ compat+0xC8 = file 0x131
        let data = make_standard_d2s_header(6, 0x00, 12, "TestChar");
        let h = D2SHeader::from_bytes(&data).unwrap();
        assert_eq!(h.class, 6);
        assert_eq!(h.character_class(), CharacterClass::Assassin);
        assert!(!h.is_hardcore());
        assert!(!h.is_expansion());
        assert_eq!(h.name, "TestChar");
    }

    #[test]
    fn test_parse_invalid_magic() {
        // 用错 magic 测试拒绝
        let mut data = b"XXX\0rest of data".to_vec();
        // 补足到 character section 长度
        data.resize(20 + 387, 0);
        let err = D2SHeader::from_bytes(&data).unwrap_err();
        assert!(matches!(err, ParseError::D2SMagic(_)));
    }

    #[test]
    fn test_parse_truncated() {
        // 太短 (< 20 字节 header) → Truncated
        let data = [0u8; 8];
        let err = D2SHeader::from_bytes(&data).unwrap_err();
        assert!(matches!(err, ParseError::Truncated(_)));
    }

    #[test]
    fn test_character_class_name_cn() {
        assert_eq!(CharacterClass::Amazon.name_cn(), "亚马逊");
        assert_eq!(CharacterClass::Necromancer.name_cn(), "死灵法师");
        assert_eq!(CharacterClass::Unknown(99).name_cn(), "未知");
        assert_eq!(CharacterClass::Warlock.name_en(), "Warlock");
        assert_eq!(CharacterClass::Barbarian.name_tw(), "野蠻人");
    }

    /// 构造标准 D2R v105 header 字节。
    /// class_id: 0=Amazon..7=Warlock
    /// status: bit2=HC, bit3=Dead, bit4=Expansion
    /// name: UTF-8 写入 compat+0xC8 位置
    fn make_standard_d2s_header(class_id: u8, status: u8, level: u8, name: &str) -> Vec<u8> {
        let mut data = vec![0u8; 0x200];
        data[0..4].copy_from_slice(D2S_MAGIC);
        // 0x04: header_size = 0x69
        data[4..8].copy_from_slice(&0x69u32.to_le_bytes());
        // 0x08: filesize, 0x0C: checksum (placeholder)
        // 0x10: active_weapon = 1
        data[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
        // 0x14: menu_layout
        // 0x18: class_id
        data[0x18] = class_id;
        // 0x19: status_flags
        data[0x19] = status;
        // 0x1A: num_skills
        // 0x1B: level
        data[0x1B] = level;
        // 0x20: save_timestamp
        // 0x28-0x4F: hotkeys
        // 0x50: left_mouse
        // 0x54-0x67: right_mouse
        // 0x68: end_marker
        // 0x69-0x193: compat section
        // name @ compat+0xC8 = file 0x131
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(64);
        let dest = 0x131;
        data[dest..dest+copy_len].copy_from_slice(&name_bytes[..copy_len]);
        if copy_len < 64 && dest + copy_len < data.len() {
            data[dest + copy_len] = 0;
        }
        data
    }

    /// 用真实 standard_test_warlock_tc03.d2s 验证标准 header 解析。
    #[test]
    fn test_real_standard_d2s_header() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("standard_test_warlock_tc03.d2s");
        if !path.exists() { eprintln!("SKIP: fixture standard_test_warlock_tc03.d2s 缺失"); return; }
        let data = std::fs::read(path).unwrap();
        let h = D2SHeader::from_bytes(&data).expect("standard d2s should parse");
        assert_eq!(h.version_raw, 105, "header_size=0x69=105");
        assert_eq!(h.class, 7, "Warlock");
        assert_eq!(h.name, "TestWarlock", "优先取 file+0x12B mod 扩展名");
        // status_flags 应包含 expansion
        assert!(h.is_expansion(), "D2R 角色始终 expansion");
        assert!(!h.is_hardcore());
    }
}
