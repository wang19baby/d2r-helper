//! D2 协议通用字段（d2i/d2s/d2x 共享）。

pub mod item;
pub mod item_flags;
pub mod item_location;
pub mod item_mode;
pub mod item_page;
pub mod item_quality;
pub mod stat;
pub mod stat_list;
pub mod stat_table;
pub mod version_dispatch;

pub use item::Item;
pub use item_flags::ItemFlags;
pub use item_location::ItemLocation;
pub use item_mode::ItemMode;
pub use item_page::ItemPage;
pub use item_quality::ItemQuality;
pub use stat::ItemStat;
pub use stat_list::{StatList, STAT_LIST_TERMINATOR};
pub use stat_table::{StatProp, StatTable};
pub use version_dispatch::FieldSet;