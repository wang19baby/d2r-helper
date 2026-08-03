//! Item page（3b）：★ 关键修复字段 ★
//!
//! D2SLib `Items.cs:283` `item.Page = reader.ReadByte(3)`：
//! - 0: Equipped
//! - 1: Inventory (Backpack)
//! - 2..4: Mod-specific
//! - 5: My Stash (D2R 私人仓库页签)
//! - 6: Shared Stash (D2R 共享仓库页签)
//! - 7: Mod-specific
//!
//! 此字段**必须读取**——之前的 `read_single_item` 漏掉它，导致后续 ~33 个装备位流错位。

use crate::core::BitReader;
use crate::core::ParseResult;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemPage {
    Equipped = 0,
    Backpack = 1,
    MyStash = 5,
    SharedStash = 6,
    Mod(u8),
}

impl ItemPage {
    pub fn read(reader: &mut BitReader) -> ParseResult<Self> {
        let v = reader.read_u8(3);
        Ok(match v {
            0 => Self::Equipped,
            1 => Self::Backpack,
            5 => Self::MyStash,
            6 => Self::SharedStash,
            2 | 3 | 4 | 7 => Self::Mod(v),
            _ => unreachable!("3 bits read; max value is 7"),
        })
    }

    /// 数字值（用于日志和回写）
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Equipped => 0,
            Self::Backpack => 1,
            Self::MyStash => 5,
            Self::SharedStash => 6,
            Self::Mod(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_item_page_my_stash() {
        // v=5 = 0b101, LSB-first → bit0=1 bit1=0 bit2=1 → byte = 0b00000101 = 0x05
        let data = [0b00000101u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemPage::read(&mut reader).unwrap(), ItemPage::MyStash);
    }

    #[test]
    fn test_read_item_page_shared_stash() {
        // v=6 = 0b110, LSB-first → bit0=0 bit1=1 bit2=1 → byte = 0b00000110 = 0x06
        let data = [0b00000110u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemPage::read(&mut reader).unwrap(), ItemPage::SharedStash);
    }

    #[test]
    fn test_read_item_page_equipped() {
        let data = [0u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemPage::read(&mut reader).unwrap(), ItemPage::Equipped);
    }

    #[test]
    fn test_read_item_page_mod_value_2() {
        // v=2 = 0b010, LSB-first → bit0=0 bit1=1 bit2=0 → byte = 0b00000010 = 0x02
        let data = [0b00000010u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemPage::read(&mut reader).unwrap(), ItemPage::Mod(2));
    }

    #[test]
    fn test_as_u8_roundtrip() {
        for v in [ItemPage::Equipped, ItemPage::Backpack, ItemPage::MyStash, ItemPage::SharedStash] {
            let mut reader = BitReader::new(&[v.as_u8()]);
            let read = ItemPage::read(&mut reader).unwrap();
            assert_eq!(read, v);
            assert_eq!(read.as_u8(), v.as_u8());
        }
    }
}