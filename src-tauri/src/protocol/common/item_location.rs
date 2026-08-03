//! Item location（4b）：物品在角色/仓库上的位置。
//!
//! D2SLib `Items.cs:265`：
//! - 0: None
//! - 1: Head
//! - 2: Neck
//! - 3: Torso
//! - 4: RightHand
//! - 5: LeftHand
//! - 6: RightFinger
//! - 7: LeftFinger
//! - 8: Waist
//! - 9: Feet
//! - 10: Hands
//! - 11: Trinket1 (D2R)
//! - 12: Trinket2 (D2R)

use crate::core::BitReader;
use crate::core::ParseResult;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemLocation {
    None = 0,
    Head = 1,
    Neck = 2,
    Torso = 3,
    RightHand = 4,
    LeftHand = 5,
    RightFinger = 6,
    LeftFinger = 7,
    Waist = 8,
    Feet = 9,
    Hands = 10,
    Trinket1 = 11,
    Trinket2 = 12,
    Unknown(u8),
}

impl ItemLocation {
    pub fn read(reader: &mut BitReader) -> ParseResult<Self> {
        let v = reader.read_u8(4);
        Ok(match v {
            0 => Self::None,
            1 => Self::Head,
            2 => Self::Neck,
            3 => Self::Torso,
            4 => Self::RightHand,
            5 => Self::LeftHand,
            6 => Self::RightFinger,
            7 => Self::LeftFinger,
            8 => Self::Waist,
            9 => Self::Feet,
            10 => Self::Hands,
            11 => Self::Trinket1,
            12 => Self::Trinket2,
            _ => Self::Unknown(v),
        })
    }

    /// 转 u8(用于 JSON 序列化 / spec §3.7 P0 装备过滤)。
    /// 由于 enum 有 field variant (`Unknown(u8)`),Rust 1.66+ 不允许 `as u8` 直转,
    /// 这里显式 match。
    pub fn as_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Head => 1,
            Self::Neck => 2,
            Self::Torso => 3,
            Self::RightHand => 4,
            Self::LeftHand => 5,
            Self::RightFinger => 6,
            Self::LeftFinger => 7,
            Self::Waist => 8,
            Self::Feet => 9,
            Self::Hands => 10,
            Self::Trinket1 => 11,
            Self::Trinket2 => 12,
            Self::Unknown(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_item_location_stored() {
        // v=10 = 0b1010, LSB-first → byte = 0b00001010 = 0x0A
        let data = [0b00001010u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemLocation::read(&mut reader).unwrap(), ItemLocation::Hands);
    }
}