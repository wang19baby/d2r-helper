//! Item flags（32b header）。
//!
//! D2SLib `Items.cs:267-289` 字段规范，MSB-first 位编号：
//! （实际 stash 格式为 LSB-first 顺序读，以下位号 = 从 flags 起始的 bit 序号）
//!
//! bits   0-3 : unknown (4b, always 0x0?)
//! bit    4   : identified
//! bits   5-10: unknown (6b)
//! bit   11   : socketed (★ NOT bit 2 ★)
//! bit   12   : unknown
//! bit   13   : "new" flag
//! bits  14-15: unknown
//! bit   16   : is_ear
//! bit   17   : starter_item
//! bits  18-20: unknown
//! bit   21   : simple_item (★ NOT bit 4 ★ — 这是关键修复)
//! bit   22   : ethereal
//! bit   23   : unknown
//! bit   24   : personalized
//! bit   25   : unknown
//! bit   26   : given_runeword
//! bits  27-31: unknown
//!
//! ★ 注意 ★：ItemFlags 的 bit 位置与 stash 文件格式强相关，
//! 来源于 `protocol::d2i::legacy::item::read_single_item` 的顺序读。
//! 新 `protocol::d2i::parser` 使用此 struct，bit 位置必须与之匹配。

use crate::core::BitReader;
use crate::core::ParseResult;
use serde::{Deserialize, Serialize};

/// 32-bit item flags header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ItemFlags {
    pub raw: u32,
}

impl ItemFlags {
    pub fn read(reader: &mut BitReader) -> ParseResult<Self> {
        Ok(Self { raw: reader.read_u32(32) })
    }

    #[inline]
    pub fn identified(&self) -> bool {
        self.raw & (1 << 4) != 0
    }

    #[inline]
    pub fn socketed(&self) -> bool {
        self.raw & (1 << 11) != 0
    }

    /// "new" flag (bit 13)
    #[inline]
    pub fn is_new(&self) -> bool {
        self.raw & (1 << 13) != 0
    }

    /// Ear of character (PvP trophy) (bit 16)
    #[inline]
    pub fn is_ear(&self) -> bool {
        self.raw & (1 << 16) != 0
    }

    /// Starter item (bit 17)
    #[inline]
    pub fn starter_item(&self) -> bool {
        self.raw & (1 << 17) != 0
    }

    #[inline]
    pub fn simple_item(&self) -> bool {
        self.raw & (1 << 21) != 0
    }

    #[inline]
    pub fn ethereal(&self) -> bool {
        self.raw & (1 << 22) != 0
    }

    #[inline]
    pub fn is_runeword(&self) -> bool {
        self.raw & (1 << 26) != 0 // same as given_runeword in stash format
    }

    #[inline]
    pub fn personalized(&self) -> bool {
        self.raw & (1 << 24) != 0
    }

    #[inline]
    pub fn given_runeword(&self) -> bool {
        self.raw & (1 << 26) != 0
    }

    #[inline]
    pub fn has_multiple_graphics(&self) -> bool {
        false // not used in stash file format
    }

    #[inline]
    pub fn unidentified(&self) -> bool {
        false // not used in stash file format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_item_flags_zero() {
        let data = [0x00u8; 4];
        let mut reader = BitReader::new(&data);
        let flags = ItemFlags::read(&mut reader).unwrap();
        assert_eq!(flags.raw, 0);
        assert!(!flags.identified());
        assert!(!flags.socketed());
        assert!(!flags.ethereal());
        assert!(!flags.is_runeword());
    }

    #[test]
    fn test_read_item_flags_identified_non_simple() {
        // stash format:
        // bits 0-3: unknown (0)
        // bit  4 : identified = 1
        // bits 5-10: unknown (0)
        // bit 11 : socketed = 0
        // bit 21 : simple_item = 0
        // => byte 0 = 0b00010000 = 0x10 (bit 4 = identified)
        // => byte 2 bit 5 = 0 (simple_item at bit 21 = 0)
        let data = [0x10u8, 0x00, 0x00, 0x00];
        let mut reader = BitReader::new(&data);
        let flags = ItemFlags::read(&mut reader).unwrap();
        assert!(flags.identified());
        assert!(!flags.simple_item());
    }

    #[test]
    fn test_read_item_flags_simple() {
        // stash format:
        // bit 21 : simple_item = 1 → byte 2 bit 5 = 0x20
        // bits 0-20, 22-31 = 0
        let data = [0x00u8, 0x00, 0x20, 0x00]; // 0x00200000 LE = bit 21
        let mut reader = BitReader::new(&data);
        let flags = ItemFlags::read(&mut reader).unwrap();
        assert!(flags.simple_item());
        assert!(!flags.identified());
    }

    #[test]
    fn test_read_item_flags_socketed_ethereal() {
        // stash format:
        // bit 11 : socketed = 1 → byte 1 bit 3 = 0x08
        // bit 22 : ethereal = 1 → byte 2 bit 6 = 0x40
        let data = [0x00u8, 0x08, 0x40, 0x00];
        let mut reader = BitReader::new(&data);
        let flags = ItemFlags::read(&mut reader).unwrap();
        assert!(flags.socketed());
        assert!(flags.ethereal());
        assert!(!flags.identified());
    }
}