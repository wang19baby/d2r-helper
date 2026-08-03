//! Item 完整聚合定义（★ Step 3 核心 ★）
//!
//! 字段顺序严格按 D2SLib `Items.cs:267-440`：
//! 1. flags (32b) → `ItemFlags`
//! 2. version (3b/10b) → `ProtocolVersion` raw 值
//! 3. mode (3b) → `ItemMode`
//! 4. location (4b) → `ItemLocation`
//! 5. x (4b), y (4b)
//! 6. page (3b) → `ItemPage` (★ 修复漏读字段 ★)
//! 7. code (4 chars Huffman)
//! 8. socket_count (1b/3b)
//! 9. 后续：id (32b), level (7b), quality (4b), ...
//!
//! 当前实现仅做聚合数据结构的骨架（Step 3），完整 read/write 留待 Step 5 重写 d2i parser。

use super::item_flags::ItemFlags;
use super::item_location::ItemLocation;
use super::item_mode::ItemMode;
use super::item_page::ItemPage;
use super::item_quality::ItemQuality;
use super::stat_list::StatList;
use crate::core::ProtocolVersion;

/// 完整 Item 聚合（d2i/d2s/d2x 通用）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub flags: ItemFlags,
    pub version_raw: u8,
    pub mode: ItemMode,
    pub location: ItemLocation,
    pub x: u8,
    pub y: u8,
    /// ★ Page 3b 字段（v97+ 才有）★ — 修复漏读 bug
    pub page: Option<ItemPage>,
    pub code: String,
    pub num_sockets: u8,
    pub id: u32,
    pub item_level: u8,
    pub quality: ItemQuality,
    /// magical properties（默认空，避免破坏既有测试）
    pub stat_lists: Vec<StatList>,
    /// 当前耐久度（武器/护甲）
    pub current_durability: u8,
    /// 最大耐久度（武器/护甲）
    pub max_durability: u8,
    /// 实际防御值（护甲/盾牌，来自 item body 11-bit defense 字段；0=非护甲或未知）
    pub defense: u16,
    /// 数量（stackable 用）
    pub amount: u32,
    pub socketed_items: Vec<Item>,
    /// 暗金物品 ID（quality=7 时有值）
    pub unique_id: Option<u16>,
    /// 套装物品 ID（quality=5 时有值）
    pub set_id: Option<u16>,
}

impl Item {
    pub fn protocol_version(&self) -> ProtocolVersion {
        ProtocolVersion::from_u32(self.version_raw as u32).unwrap_or(ProtocolVersion::CURRENT)
    }

    /// 返回此 item 总数(含 socketed 子物品)
    pub fn total_count(&self) -> usize {
        1 + self.socketed_items.iter().map(|i| i.total_count()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_protocol_version() {
        let item = Item {
            flags: ItemFlags::default(),
            version_raw: 105,
            mode: ItemMode::Stored,
            location: ItemLocation::None,
            x: 0,
            y: 0,
            page: Some(ItemPage::MyStash),
            code: "r01".into(),
            num_sockets: 0,
            id: 0,
            item_level: 1,
            quality: ItemQuality::Normal,
            stat_lists: Vec::new(),
            amount: 1,
            socketed_items: Vec::new(),
            unique_id: None,
            set_id: None,
            current_durability: 0,
            max_durability: 0,
            defense: 0,
        };
        assert_eq!(item.protocol_version(), ProtocolVersion::V105);
        assert_eq!(item.page, Some(ItemPage::MyStash));
    }

    #[test]
    fn test_item_default_page_is_some_for_v105() {
        let item = Item {
            flags: ItemFlags::default(),
            version_raw: 105,
            mode: ItemMode::Equipped,
            location: ItemLocation::Head,
            x: 5,
            y: 3,
            page: Some(ItemPage::Equipped),
            code: "ba5".into(),
            num_sockets: 0,
            id: 0,
            item_level: 10,
            quality: ItemQuality::Normal,
            stat_lists: Vec::new(),
            amount: 1,
            socketed_items: Vec::new(),
            unique_id: None,
            set_id: None,
            current_durability: 0,
            max_durability: 0,
            defense: 0,
        };
        assert_eq!(item.page, Some(ItemPage::Equipped));
        assert_eq!(item.location, ItemLocation::Head);
    }
}