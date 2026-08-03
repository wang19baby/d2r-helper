//! StatList 容器（0x1FF 终止符循环 + sub-property 自动展开）。

use serde::{Deserialize, Serialize};
use crate::core::BitReader;
use crate::core::ParseResult;
use crate::protocol::common::stat::ItemStat;
use crate::protocol::common::stat_table::StatTable;
use crate::protocol::common::version_dispatch::FieldSet;

/// 终止符常量（D2SLib MagicStatList）。
pub const STAT_LIST_TERMINATOR: u16 = 0x1FF;

/// Configuration for reading stat streams with safety guards.
///
/// Item stats and character stats have different constraints:
/// items have bounded stat counts while character stats are effectively unlimited.
#[derive(Debug, Clone)]
pub struct StatReadConfig {
    /// Max total stats before termination (100 for items, unlimited for character).
    pub max_stats: u32,
    /// Max occurrences of the same stat ID before terminating.
    pub max_same_stat: u32,
    /// Max consecutive unknown (ID > known_id_max) stats before terminating.
    pub max_unknown_consec: u32,
    /// Max consecutive core character stats before terminating.
    pub max_core_consec: u32,
    /// Default save_bits when prop.save_bits == 0.
    pub default_save_bits: u8,
    /// Highest known stat ID. IDs above this are "unknown".
    pub known_id_max: u16,
    /// Stat IDs that are in range but unknown.
    pub unknown_ids: &'static [u16],
    /// Core character stat IDs (shouldn't appear on items).
    pub core_stats: &'static [u16],
}

impl Default for StatReadConfig {
    /// Item-appropriate defaults.
    fn default() -> Self {
        static UNKNOWN: &[u16] = &[212, 375, 376, 377, 378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 388, 389, 390, 391, 392, 393, 394, 395];
        Self {
            max_stats: 100,
            max_same_stat: 8,
            max_unknown_consec: 1,
            max_core_consec: 3,
            default_save_bits: 9,
            known_id_max: 419,
            unknown_ids: UNKNOWN,
            core_stats: &[4, 5, 6, 8, 10, 12, 13, 14, 15],
        }
    }
}

impl StatReadConfig {
    /// Config for character stats — unlimited stat count, no core-stat guard.
    pub fn character_defaults() -> Self {
        Self {
            max_stats: u32::MAX,
            max_core_consec: u32::MAX,
            ..Self::default()
        }
    }
}

/// 一组 stat。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatList {
    pub stats: Vec<ItemStat>,
}

impl StatList {
    /// Read stat list with default item-appropriate guard configuration.
    pub fn read(reader: &mut BitReader, table: &StatTable) -> ParseResult<Self> {
        Self::read_with_config(reader, table, &StatReadConfig::default())
    }

    /// Read stat list with custom guard configuration.
    pub fn read_with_config(reader: &mut BitReader, table: &StatTable, config: &StatReadConfig) -> ParseResult<Self> {
        let mut stats = Vec::new();
        let mut consec_unknown = 0u32;
        let mut consec_core = 0u32;
        let mut stat_counts: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
        loop {
            if reader.remaining_bits() < 9 || stats.len() as u32 >= config.max_stats {
                break;
            }
            let id = reader.read_u16(9);
            if id == STAT_LIST_TERMINATOR {
                break;
            }
            // Guard: same stat ID appearing too many times
            let count = stat_counts.entry(id).or_insert(0);
            *count += 1;
            if *count >= config.max_same_stat { break; }
            // Guard: consecutive core character stats (shouldn't be on items)
            if config.core_stats.contains(&id) {
                consec_core += 1;
                consec_unknown = 0;
                if consec_core >= config.max_core_consec { break; }
            } else {
                consec_core = 0;
                // Guard: consecutive unknown stat IDs (matching Python _ALL_STAT_IDS)
                if id > config.known_id_max || config.unknown_ids.contains(&id) {
                    consec_unknown += 1;
                    if consec_unknown >= config.max_unknown_consec { break; }
                } else {
                    consec_unknown = 0;
                }
            }
            let prop = table.get(id);
            match ItemStat::read(reader, id, &prop) {
                Ok(stat) => stats.push(stat),
                Err(_) => break,
            }
            let sub_count = FieldSet::sub_property_count(id);
            for offset in 1..=sub_count as u16 {
                let sub_id = id + offset;
                let sub_prop = table.get(sub_id);
                if let Ok(stat) = ItemStat::read(reader, sub_id, &sub_prop) {
                    stats.push(stat);
                } else {
                    break;
                }
            }
        }
        Ok(Self { stats })
    }
    /// Read stat list and return (list, stat_clean) where stat_clean=true means
    /// the stream terminated by finding 0x1FF (vs. guard/max/remainder termination).
    /// Matches Python's _scan_stat_stream which returns (bits, terminated_by_0x1FF).
    pub fn read_with_clean_flag(reader: &mut BitReader, table: &StatTable, config: &StatReadConfig) -> (Self, bool) {
        let mut stats = Vec::new();
        let mut consec_unknown = 0u32;
        let mut consec_core = 0u32;
        let mut stat_counts: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
        let mut terminated_by_0x1ff = false;
        loop {
            if reader.remaining_bits() < 9 || stats.len() as u32 >= config.max_stats {
                break;
            }
            let id = reader.read_u16(9);
            if id == STAT_LIST_TERMINATOR {
                terminated_by_0x1ff = true;
                break;
            }
            // Same guards as read_with_config
            let count = stat_counts.entry(id).or_insert(0);
            *count += 1;
            if *count >= config.max_same_stat { break; }
            if config.core_stats.contains(&id) {
                consec_core += 1;
                consec_unknown = 0;
                if consec_core >= config.max_core_consec { break; }
            } else {
                consec_core = 0;
                if id > config.known_id_max || config.unknown_ids.contains(&id) {
                    consec_unknown += 1;
                    if consec_unknown >= config.max_unknown_consec { break; }
                } else {
                    consec_unknown = 0;
                }
            }
            let prop = table.get(id);
            match ItemStat::read(reader, id, &prop) {
                Ok(stat) => stats.push(stat),
                Err(_) => break,
            }
            let sub_count = FieldSet::sub_property_count(id);
            for offset in 1..=sub_count as u16 {
                let sub_id = id + offset;
                let sub_prop = table.get(sub_id);
                if let Ok(stat) = ItemStat::read(reader, sub_id, &sub_prop) {
                    stats.push(stat);
                } else {
                    break;
                }
            }
        }
        (Self { stats }, terminated_by_0x1ff)
    }

    /// Write all stats followed by the terminator (0x1FF).
    pub fn write(&self, writer: &mut crate::core::bitio::BitWriter, table: &StatTable) {
        for stat in &self.stats {
            let prop = table.get(stat.id);
            stat.write(writer, &prop);
            let sub_count = crate::protocol::common::version_dispatch::FieldSet::sub_property_count(stat.id);
            for offset in 1..=sub_count as u16 {
                let sub_prop = table.get(stat.id + offset);
                stat.write(writer, &sub_prop);
            }
        }
        writer.write_u16(STAT_LIST_TERMINATOR, 9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::common::stat_table::StatProp;

    fn empty_table() -> StatTable {
        StatTable::from_props(vec![StatProp::default(); 512])
    }

    #[test]
    fn test_terminator_only() {
        // 0x1FF = 511 in 9 bits = 0b111111111
        let data = [0xFF, 0x01];
        let mut reader = BitReader::new(&data);
        let list = StatList::read(&mut reader, &empty_table()).unwrap();
        assert!(list.stats.is_empty());
    }

    #[test]
    fn test_single_stat_then_terminator() {
        // id=0 (9b) = 0b000000000, save_bits=8 → value consumes 8 bits
        // terminator: 0x1FF (9b) = 0b111111111
        // 总 9 + 8 + 9 = 26 bits
        let bits: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, // id = 0
            0, 0, 1, 0, 0, 0, 0, 0,    // value = 0x20 (LSB first 8 bits)
            1, 1, 1, 1, 1, 1, 1, 1, 1, // terminator = 0x1FF
            0, 0, 0, 0, 0, 0,           // padding to 32 bits
        ];
        let mut bytes = vec![0u8; 4];
        for (i, bit) in bits.iter().enumerate() {
            if *bit != 0 {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        let mut reader = BitReader::new(&bytes);
        // Use table with save_bits=8 for the stat at id=0
        let mut props = vec![StatProp::default(); 512];
        props[0] = StatProp { save_bits: 8, ..Default::default() };
        let table = StatTable::from_props(props);
        let list = StatList::read(&mut reader, &table).unwrap();
        assert_eq!(list.stats.len(), 1);
        assert_eq!(list.stats[0].id, 0);
    }

    #[test]
    fn test_two_stats_then_terminator() {
        // id=0 (9b), value=0 (8b), id=1 (9b), value=0 (8b), terminator (9b) = 43 bits
        let bits: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, // id=0 (9b)
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0 (8b)
            1, 0, 0, 0, 0, 0, 0, 0, 0, // id=1 (9b)
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0 (8b)
            1, 1, 1, 1, 1, 1, 1, 1, 1, // terminator (9b)
            0, 0, 0, 0, 0,              // padding to 48 bits
        ];
        let mut bytes = vec![0u8; 6];
        for (i, bit) in bits.iter().enumerate() {
            if *bit != 0 {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        let mut reader = BitReader::new(&bytes);
        let mut props = vec![StatProp::default(); 512];
        props[0] = StatProp { save_bits: 8, ..Default::default() };
        props[1] = StatProp { save_bits: 8, ..Default::default() };
        let table = StatTable::from_props(props);
        let list = StatList::read(&mut reader, &table).unwrap();
        assert_eq!(list.stats.len(), 2);
        assert_eq!(list.stats[0].id, 0);
        assert_eq!(list.stats[1].id, 1);
    }

    #[test]
    fn test_read_with_config_character_defaults() {
        // 5 consecutive core stats followed by terminator — should pass with character config
        // but be rejected by default item config
        let bits: Vec<u8> = vec![
            // id=4 (Str, 9b), value=0 (8b) × 5
            0, 0, 1, 0, 0, 0, 0, 0, 0, // id=4
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0
            0, 0, 1, 0, 0, 0, 0, 0, 0, // id=4
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0
            0, 0, 1, 0, 0, 0, 0, 0, 0, // id=4
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0
            0, 0, 1, 0, 0, 0, 0, 0, 0, // id=4
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0
            0, 0, 1, 0, 0, 0, 0, 0, 0, // id=4 (5th — would trigger item guard)
            0, 0, 0, 0, 0, 0, 0, 0,    // value=0
            1, 1, 1, 1, 1, 1, 1, 1, 1, // terminator
        ];
        let mut bytes = vec![0u8; 13];
        for (i, bit) in bits.iter().enumerate() {
            if *bit != 0 {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        let mut props = vec![StatProp::default(); 512];
        props[4] = StatProp { save_bits: 8, ..Default::default() };
        let table = StatTable::from_props(props);

        // Default (item) config: 3 consecutive core stats → break early
        let mut r1 = BitReader::new(&bytes);
        let item_result = StatList::read(&mut r1, &table).unwrap();
        assert!(item_result.stats.len() < 5, "item config should break after 3 core stats, got {}", item_result.stats.len());

        // Character config: unlimited consecutive core stats → reads all 5
        let mut r2 = BitReader::new(&bytes);
        let char_result = StatList::read_with_config(&mut r2, &table, &StatReadConfig::character_defaults()).unwrap();
        assert_eq!(char_result.stats.len(), 5, "character config should read all 5 core stats");
    }
}