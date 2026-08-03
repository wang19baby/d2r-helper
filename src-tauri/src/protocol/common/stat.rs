//! ItemStat 单个属性。
//!
//! D2SLib 字段顺序（紧跟 stat_id 之后）：
//! 1. param（位宽 = `prop.save_param_bits`），某些 stat 用作 skill/state/monster ID
//! 2. value（位宽 = `prop.save_bits`），可能 signed
//! 3. apply `save_add` 偏移（save_add 在读取时减去，写入时加上）
//!
//! ## 特殊编码（D2SLib `ItemStat.Read` 行为）
//!
//! - `prop.descfunc == 14`（仅 stat 188 `item_addskill_tab`）：
//!   - `SkillTab = param & 0x7`          (低 3 bits = tab_within_class)
//!   - `SkillLevel = (param >> 3) & 0x1fff` (高 13 bits = +N skill levels)
//!
//! - `prop.encoding == 2`（chance to cast skill, e.g. "+X% chance to cast Y on strike"）：
//!   - `SkillLevel = param & 0x3f`        (低 6 bits)
//!   - `SkillId = (param >> 6) & 0x3ff`   (高 10 bits)
//!
//! - `prop.encoding == 3`（skill charges, e.g. "+N charges of skill X"）：
//!   - `SkillLevel = param & 0x3f`        (低 6 bits, unused 保留)
//! - `prop.encoding == 2`（chance to cast skill, e.g. "+X% chance to cast Y on strike"）：
//!   - `SkillLevel = param & 0x3f`        (低 6 bits)
//!   - `SkillId = (param >> 6) & 0x3ff`   (高 10 bits)
//!   - `Value = SaveBits - SaveAdd`       (触发概率%)
//!
//! - `prop.encoding == 3`（skill charges, e.g. "+N charges of skill X"）：
//!   - `SkillLevel = param & 0x3f`        (低 6 bits, unused 保留)
//!   - `SkillId = (param >> 6) & 0x3ff`   (高 10 bits)
//!   - `MaxCharges = (raw_value >> 8) & 0xff` (高 8 bits of value)
//!   - `Value = raw_value & 0xff`         (低 8 bits of value = current charges)
//!
//! - `prop.encoding == 1`（item_singleskill, e.g. "+N to Skill X"）：
//!   - `SkillId = SaveParam`              (param 就是 skill_id)
//!   - `SkillLevel = SaveBits - SaveAdd`  (value 就是 skill_level)
//!
//! - `prop.encoding == 0 / 4`：无 skill 拆分, param/value 保持原始值

use serde::{Deserialize, Serialize};
use crate::core::BitReader;
use crate::core::ParseResult;
use crate::protocol::common::stat_table::StatProp;

/// 单个 stat（id + param + value + 可选 skill 拆分字段）。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ItemStat {
    pub id: u16,
    /// param 字段（skill ID / state ID / 拆分前的 raw）
    pub param: u32,
    /// raw value（解码后减去 save_add）
    pub value: i64,
    /// descfunc=14: tab_within_class (3 bits, 0-7)。例：`+1 暴风雪` 的 tab=0 (Sorceress Cold)
    pub skill_tab: Option<u8>,
    /// encode=2/3: skill level (6 bits, 0-63)。
    /// descfunc=14: skill level value (13 bits, 0-8191)。
    pub skill_level: Option<u16>,
    /// encode=2/3: skill id (10 bits, 0-1023)。
    pub skill_id: Option<u16>,
    /// encode=3: max charges (8 bits)。
    pub max_charges: Option<u8>,
}

impl ItemStat {
    /// 按给定 prop 表读取一个 stat。
    ///
    /// 完整实现 D2SLib `ItemStat.Read()` 的所有 encode/descfunc 分支。
    pub fn read(reader: &mut BitReader, id: u16, prop: &StatProp) -> ParseResult<Self> {
        // 1. 读 param 字段（位宽 = save_param_bits）
        let param_raw = if prop.save_param_bits > 0 {
            reader.read_u32(prop.save_param_bits)
        } else {
            0
        };

        // 2. 读 value 字段（位宽 = save_bits）
        let raw_value_bits = if prop.save_bits > 0 {
            reader.read_u32(prop.save_bits)
        } else {
            // Python d2r-zero defaults to 9 bits for unknown/zero-bits stats.
            // This is critical for mod stash alignment — mod stats with save_bits=0
            // in ItemStatCost.txt still consume 9 bits in the bitstream.
            reader.read_u32(9)
        };

        // 3. 拆分 param/value（按 descfunc + encoding）
        let mut stat = Self {
            id,
            param: param_raw,
            value: 0, // 后面按 encoding 调整
            skill_tab: None,
            skill_level: None,
            skill_id: None,
            max_charges: None,
        };

        // descfunc=14: stat 188 item_addskill_tab — param 低 3 位是 tab index
        // D2R itemstatcost: Send Param Bits=6, tab = param & 0x7;
        // 等级 = value 字段 (不是 param 高位, 实测 param=32/33/34 → tab 0/1/2)
        if prop.descfunc == 14 {
            stat.skill_tab = Some((param_raw & 0x7) as u8);
        }
        // encode=1: item_singleskill — param 就是 skill_id, value 就是 skill_level
        else if prop.encoding == 1 {
            stat.skill_id = Some(param_raw as u16);
            stat.skill_level = Some((raw_value_bits as i64 - prop.save_add as i64) as u16);
        }
        // encode=2/3: chance to cast / skill charges — 把 param 拆为 SkillLevel + SkillId
        else if prop.encoding == 2 || prop.encoding == 3 {
            stat.skill_level = Some((param_raw & 0x3f) as u16);
            stat.skill_id = Some(((param_raw >> 6) & 0x3ff) as u16);
        }
        // encode=0/4: 无 skill 拆分, param/value 保持原始值

        // encode=3: skill charges — 把 value 拆为 current charges (低 8 bits) + max charges (高 8 bits)
        if prop.encoding == 3 && prop.save_bits >= 16 {
            stat.max_charges = Some(((raw_value_bits >> 8) & 0xff) as u8);
            stat.value = (raw_value_bits & 0xff) as i64 - prop.save_add as i64;
        } else {
            // save_add 在写入时加上，读取时减去
            stat.value = raw_value_bits as i64 - prop.save_add as i64;
        }

        // descfunc=14: 等级 = 解析后的 value (tab 已在上面从 param 低 3 位取)
        if prop.descfunc == 14 {
            stat.skill_level = Some(stat.value.clamp(0, u16::MAX as i64) as u16);
        }

        Ok(stat)
    }
    /// 返回减去 save_add 后的显示值（兼容未调整的 stat 值）。
    /// 部分解析路径未对 base stats (0-3,7,9) 应用 save_add，
    /// 此方法在已知有 save_add 的 stat ID 上做修正。
    pub fn display_value(&self) -> i64 {
        const SAVE_ADD_ATTRS: [(u16, i64); 6] = [
            (0, 32),  // strength
            (1, 32),  // energy
            (2, 32),  // dexterity
            (3, 32),  // vitality
            (7, 32),  // maxhp
            (9, 32),  // maxmana
        ];
        for &(id, add) in &SAVE_ADD_ATTRS {
            if self.id == id && self.value > add {
                return self.value - add;
            }
        }
        self.value
    }

    /// Write this stat to a bitstream using the given property definition.
    pub fn write(&self, writer: &mut crate::core::bitio::BitWriter, prop: &StatProp) {
        if prop.save_param_bits > 0 {
            let pv = match prop.encoding {
                1 => self.skill_id.unwrap_or(self.param as u16) as u32,
                _ => self.param,
            };
            writer.write_u32(pv, prop.save_param_bits);
        }
        let n = if prop.save_bits > 0 { prop.save_bits } else { 9 };
        let v = (self.value as u32).wrapping_add(prop.save_add as u32);
        writer.write_u32(v, n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prop(save_bits: u8, save_param_bits: u8, save_add: i32, signed: u8) -> StatProp {
        StatProp {
            save_bits,
            save_param_bits,
            save_add,
            signed,
            ..Default::default()
        }
    }

    /// 完整 prop 构造器 (覆盖 encoding + descfunc)
    fn prop_full(save_bits: u8, save_param_bits: u8, save_add: i32, signed: u8, encoding: u8, descfunc: u8) -> StatProp {
        StatProp {
            save_bits,
            save_param_bits,
            save_add,
            signed,
            encoding,
            descfunc,
            ..Default::default()
        }
    }

    #[test]
    fn test_read_unsigned_stat() {
        // save_bits=8, save_add=32, signed=0 → value = raw - 32
        let data = [0x20u8]; // raw = 32
        let mut reader = BitReader::new(&data);
        let stat = ItemStat::read(&mut reader, 0, &prop(8, 0, 32, 0)).unwrap();
        assert_eq!(stat.id, 0);
        assert_eq!(stat.value, 0); // 32 - 32 = 0
    }

    #[test]
    fn test_read_unsigned_stat_with_positive_add() {
        // save_bits=9, save_add=200 → value = raw - 200
        let data = [0xC8u8]; // raw = 200
        let mut reader = BitReader::new(&data);
        let stat = ItemStat::read(&mut reader, 39, &prop(9, 0, 200, 1)).unwrap();
        assert_eq!(stat.value, 0);
    }

    #[test]
    fn test_read_stat_with_param() {
        // save_param_bits=8 + save_bits=8 → 16 bits
        // param = 0x12 = 18, value raw = 0x34 = 52
        let data = [0x12, 0x34];
        let mut reader = BitReader::new(&data);
        let stat = ItemStat::read(&mut reader, 97, &prop(8, 8, 0, 1)).unwrap();
        assert_eq!(stat.param, 18);
        assert_eq!(stat.value, 52);
    }

    #[test]
    fn test_zero_bits_stat() {
        // save_bits=0: Python d2r-zero defaults to 9 bits for alignment.
        // Even though the stat table says 0 bits, the parser reads 9 bits
        // to stay in sync with the bitstream (mod stashes require this).
        let data = [0u8];
        let mut reader = BitReader::new(&data);
        let stat = ItemStat::read(&mut reader, 359, &prop(0, 0, 0, 0)).unwrap();
        assert_eq!(stat.param, 0);
        assert_eq!(stat.value, 0);
        assert_eq!(reader.offset(), 9, "save_bits=0 defaults to 9 bits read");
    }

    // ── Phase I (2026-07-09) 修复: descfunc=14 / encode=2 / encode=3 ──
    // 2026-07-31 修正: D2R itemstatcost 188 (item_addskill_tab) Send Param Bits=6,
    // tab = param & 0x7 (param 直接是 tab index), 等级 = value 字段。
    // 实测恶魔角锋: param=32/33/34 → tab 0/1/2, value=3/2/2 = 等级。
    // (旧假设 param=(level<<3)|tab 与 D2R 不符, 已废弃。)

    /// ★ descfunc=14 (stat 188 item_addskill_tab): tab = param & 0x7, 等级 = value
    ///
    /// 模拟 "+1 暴风雪技能" (Sorceress Cold tab=0, +1 level):
    ///   id=188 (9 bits), param=0x00 (tab=0), value=1 (3 bits)
    ///   总 9+16+3 = 28 bits
    #[test]
    fn test_read_skill_tab_descfunc14() {
        let mut bits: Vec<u8> = Vec::new();
        bits.extend(num_to_lsb9(188));                       // 9 bits
        bits.extend(num_to_lsb_n(0x0000, 16));                // 16 bits (param=0 → tab 0)
        bits.extend(num_to_lsb_n(1, 3));                      // 3 bits (value=1 → +1 level)
        let bytes = bits_to_bytes(&bits);
        // 模拟完整 stat stream: reader 先读 9-bit stat_id, 然后 ItemStat::read
        let mut reader = BitReader::new(&bytes);
        let _id = reader.read_u16(9);
        let stat = ItemStat::read(&mut reader, 188, &prop_full(3, 16, 0, 1, 0, 14)).unwrap();
        assert_eq!(stat.id, 188);
        assert_eq!(stat.skill_tab, Some(0), "Sorceress Cold tab");
        assert_eq!(stat.skill_level, Some(1), "+1 level");
        assert_eq!(stat.value, 1);
        assert_eq!(stat.skill_id, None);
        assert_eq!(stat.max_charges, None);
    }

    /// ★ descfunc=14 边界: tab=7, 等级=value
    ///   param=0x0007 (tab=7), value=3 (3 bits)
    #[test]
    fn test_read_skill_tab_descfunc14_max_values() {
        let mut bits: Vec<u8> = Vec::new();
        bits.extend(num_to_lsb9(188));
        bits.extend(num_to_lsb_n(0x0007, 16));
        bits.extend(num_to_lsb_n(3, 3));
        let bytes = bits_to_bytes(&bits);
        let mut reader = BitReader::new(&bytes);
        let _id = reader.read_u16(9);
        let stat = ItemStat::read(&mut reader, 188, &prop_full(3, 16, 0, 1, 0, 14)).unwrap();
        assert_eq!(stat.skill_tab, Some(7));
        assert_eq!(stat.skill_level, Some(3));
        assert_eq!(stat.value, 3);
    }

    /// ★ encode=2 (chance to cast skill, e.g. "+5% chance to cast Blizzard"):
    ///   param 拆为 SkillLevel (低 6 bits) + SkillId (高 10 bits)
    ///
    /// 模拟 skill_id=Blizzard=64=0x40, level=0:
    ///   param = 0x40 << 6 = 0x1000
    ///   value = 5 (5%, save_bits=7)
    ///   total 9+16+7 = 32 bits
    #[test]
    fn test_read_chance_to_cast_encode2() {
        let mut bits: Vec<u8> = Vec::new();
        bits.extend(num_to_lsb9(195));
        bits.extend(num_to_lsb_n(0x1000, 16));
        bits.extend(num_to_lsb_n(5, 7));
        let bytes = bits_to_bytes(&bits);
        let mut reader = BitReader::new(&bytes);
        let _id = reader.read_u16(9);
        let stat = ItemStat::read(&mut reader, 195, &prop_full(7, 16, 0, 1, 2, 0)).unwrap();
        assert_eq!(stat.id, 195);
        assert_eq!(stat.skill_id, Some(64), "Blizzard skill id");
        assert_eq!(stat.skill_level, Some(0));
        assert_eq!(stat.value, 5, "5% chance");
        assert_eq!(stat.skill_tab, None);
        assert_eq!(stat.max_charges, None);
    }

    /// ★ encode=3 (skill charges, e.g. "+30 Blizzard charges"):
    ///   param = skill_level | (skill_id << 6)
    ///   value 拆为 current (低 8 bits) + max (高 8 bits)
    ///
    /// 模拟 skill_id=64, current=3, max=30:
    ///   param = 0x1000, value = (30 << 8) | 3 = 0x1E03
    ///   total 9+16+16 = 41 bits
    #[test]
    fn test_read_skill_charges_encode3() {
        let mut bits: Vec<u8> = Vec::new();
        bits.extend(num_to_lsb9(204));
        bits.extend(num_to_lsb_n(0x1000, 16));
        bits.extend(num_to_lsb_n(0x1E03, 16));
        let bytes = bits_to_bytes(&bits);
        let mut reader = BitReader::new(&bytes);
        let _id = reader.read_u16(9);
        let stat = ItemStat::read(&mut reader, 204, &prop_full(16, 16, 0, 1, 3, 0)).unwrap();
        assert_eq!(stat.id, 204);
        assert_eq!(stat.skill_id, Some(64));
        assert_eq!(stat.skill_level, Some(0));
        assert_eq!(stat.max_charges, Some(30), "max 30 charges");
        assert_eq!(stat.value, 3, "current 3 charges");
    }

    /// ★ encode=1 (item_singleskill, e.g. "+1 to Skill 395"):
    ///   param = skill_id, value = skill_level (无位级拆分)
    ///
    /// 模拟 dgr 匕首的 stat 107: skill_id=395, skill_level=1
    ///   param = 395 (9 bits: 0x18B)
    ///   value = 1 (3 bits)
    ///   total 9+9+3 = 21 bits
    #[test]
    fn test_read_single_skill_encode1() {
        let mut bits: Vec<u8> = Vec::new();
        bits.extend(num_to_lsb9(107));
        bits.extend(num_to_lsb_n(395, 9));
        bits.extend(num_to_lsb_n(1, 3));
        let bytes = bits_to_bytes(&bits);
        let mut reader = BitReader::new(&bytes);
        let _id = reader.read_u16(9);
        let stat = ItemStat::read(&mut reader, 107, &prop_full(3, 9, 0, 1, 1, 0)).unwrap();
        assert_eq!(stat.id, 107);
        assert_eq!(stat.skill_id, Some(395), "param IS skill_id");
        assert_eq!(stat.skill_level, Some(1), "value IS skill_level");
        assert_eq!(stat.value, 1);
        assert_eq!(stat.param, 395);
        assert_eq!(stat.skill_tab, None);
        assert_eq!(stat.max_charges, None);
    }

    /// 回归: 普通 stat (encode=0, descfunc=0) 不拆分, 保留 param + value
    #[test]
    fn test_read_no_encode_keeps_param_and_value() {
        // stat 50 (lightmindam) save_bits=6, save_param_bits=0, encoding=0
        let data = [0x05u8]; // raw = 5
        let mut reader = BitReader::new(&data);
        let stat = ItemStat::read(&mut reader, 50, &prop(6, 0, 0, 1)).unwrap();
        assert_eq!(stat.id, 50);
        assert_eq!(stat.value, 5);
        assert_eq!(stat.param, 0);
        assert_eq!(stat.skill_tab, None);
        assert_eq!(stat.skill_level, None);
        assert_eq!(stat.skill_id, None);
        assert_eq!(stat.max_charges, None);
    }

    /// helper: 把 LSB-first bit 数组转换为 byte 数组 (高位补 0)
    fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
        let byte_count = bits.len().div_ceil(8);
        let mut bytes = vec![0u8; byte_count];
        for (i, b) in bits.iter().enumerate() {
            if *b != 0 {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        bytes
    }

    /// helper: 把 u32 转为 LSB-first 9-bit 数组 (用于 stat_id)
    fn num_to_lsb9(n: u32) -> [u8; 9] {
        [
            n as u8 & 1,
            (n >> 1) as u8 & 1,
            (n >> 2) as u8 & 1,
            (n >> 3) as u8 & 1,
            (n >> 4) as u8 & 1,
            (n >> 5) as u8 & 1,
            (n >> 6) as u8 & 1,
            (n >> 7) as u8 & 1,
            (n >> 8) as u8 & 1,
        ]
    }

    /// helper: 把 u32 转为 LSB-first n-bit 数组
    fn num_to_lsb_n(n: u32, bits: u8) -> Vec<u8> {
        (0..bits).map(|i| (n >> i) as u8 & 1).collect()
    }
}