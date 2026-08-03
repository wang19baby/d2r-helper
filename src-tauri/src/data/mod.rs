//! Static data tables (game constants, item definitions, recipes).
//!
//! 内容从 `protocol::d2i::legacy::*` 逐步迁入：
//! - `stat_cost`：itemstatcost.txt stat 表
//! - `items`：ITEM_CODE_MAP / ITEM_NAME_TO_CODE / STACKABLE_ITEM_CODES（re-export）

pub mod items;
pub mod items_base;
pub mod runewords;
pub mod python_stats;
pub mod waypoints_zh;
pub mod quests_zh;
pub mod skills_zh;
pub mod affixes_zh;
pub mod item_names_zh;
pub mod modifiers_zh;
pub mod stat_cost;
pub mod affix_names;
pub mod stat_loader;
pub mod builds;