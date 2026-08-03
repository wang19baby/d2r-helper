//! Item quality（4b）。
//!
//! D2SLib `Items.cs:284` `item.Quality = reader.ReadByte(4)`：
//! - 0: None (invalid)
//! - 1: Low Quality
//! - 2: Normal
//! - 3: Superior
//! - 4: Magic
//! - 5: Set
//! - 6: Rare
//! - 7: Unique
//! - 8: Crafted

use crate::core::BitReader;
use crate::core::ParseResult;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemQuality {
    None = 0,
    Low = 1,
    Normal = 2,
    Superior = 3,
    Magic = 4,
    Set = 5,
    Rare = 6,
    Unique = 7,
    Crafted = 8,
    Unknown(u8),
}

impl ItemQuality {
    pub fn read(reader: &mut BitReader) -> ParseResult<Self> {
        let v = reader.read_u8(4);
        Ok(match v {
            0 => Self::None,
            1 => Self::Low,
            2 => Self::Normal,
            3 => Self::Superior,
            4 => Self::Magic,
            5 => Self::Set,
            6 => Self::Rare,
            7 => Self::Unique,
            8 => Self::Crafted,
            _ => Self::Unknown(v),
        })
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Low => 1,
            Self::Normal => 2,
            Self::Superior => 3,
            Self::Magic => 4,
            Self::Set => 5,
            Self::Rare => 6,
            Self::Unique => 7,
            Self::Crafted => 8,
            Self::Unknown(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_item_quality_unique() {
        // v=7 = 0b0111, LSB-first → byte = 0b00000111 = 0x07
        let data = [0b00000111u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemQuality::read(&mut reader).unwrap(), ItemQuality::Unique);
    }

    #[test]
    fn test_read_item_quality_set() {
        // v=5 = 0b0101, LSB-first → byte = 0b00000101 = 0x05
        let data = [0b00000101u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemQuality::read(&mut reader).unwrap(), ItemQuality::Set);
    }

    #[test]
    fn test_read_item_quality_normal() {
        // v=2 = 0b0010, LSB-first → byte = 0b00000010 = 0x02
        let data = [0b00000010u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemQuality::read(&mut reader).unwrap(), ItemQuality::Normal);
    }
}