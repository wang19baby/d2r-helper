//! D2S 角色存档协议层（基础骨架）。
//!
//! 模块结构：
//! - `header`：文件 magic + version + name + status/class
//! - `attributes`：基础属性（力量/敏捷/体力/能量 + 6 misc）
//! - `items`：items 段 bit-level JM 解析（标准 D2S）
//! - `items_modified`：魔改 layout（d2emu 仙道 mod）解析
//! - `parser`：顶层 parse_file 入口
//!
//! 后续扩展：corpse / waypoints / quests / player_stats。

pub mod attributes;
pub mod header;
pub mod items;
pub mod items_modified;
pub mod magic_affix;
pub mod parser;

pub use attributes::{parse as parse_attributes, parse_with_table, AttributeId, CharacterAttributes};
pub use header::{CharacterClass, D2SHeader, D2S_MAGIC};
pub use parser::{
    parse_file, parse_skills,
    known_item_bit_layout, D2SCharacter,
    KnownItemBitLayout, SkillEntry, WaypointSet, QuestEntry,
    WooQuestData, W4DialogData,
};

/// Attributes 段在 d2s 文件中的固定起始 offset (xieedi.d2s + 开心的蛮蛮.d2s 实测一致)。
pub const ATTRIBUTES_OFFSET: usize = 0x341;
