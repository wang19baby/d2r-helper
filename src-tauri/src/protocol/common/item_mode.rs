//! Item mode（3b）：物品状态机。
//!
//! ⚠️ 数值口径（2026-08-02 修正）：
//! `Socket = 6` 是 **D2R 实测值**——TC59 StressTest.d2i 中 4 个 socketed
//! 子物品（jew）全部被 `associate_socketed_items` 按 `mode == Socket` 吸收，
//! 而该枚举值只由 `jm_reader::u8_to_mode(6)` 产生，反证真实位流 mode=6。
//!
//! 经典 D2 / D2SLib 旧版枚举把 `Socket` 放在 4（`4=Socket, 5/6=Unused`），
//! 与 D2R v105 位流不符，不要改回。`Buffer=3` 沿用 D2SLib，未经 D2R 实测。
//!
//! - 0: Stored
//! - 1: Equipped
//! - 2: Belt
//! - 3: Buffer（D2SLib 口径，未实测）
//! - 4: Unused
//! - 5: Unused
//! - 6: Socket（D2R 实测）
//! - 7: Unknown

use crate::core::BitReader;
use crate::core::ParseResult;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemMode {
    Stored = 0,
    Equipped = 1,
    Belt = 2,
    Buffer = 3,
    Socket = 4,
    Unused5 = 5,
    Unused6 = 6,
    Unknown = 7,
}

impl ItemMode {
    pub fn read(reader: &mut BitReader) -> ParseResult<Self> {
        let v = reader.read_u8(3);
        Ok(match v {
            0 => Self::Stored,
            1 => Self::Equipped,
            2 => Self::Belt,
            3 => Self::Buffer,
            4 => Self::Socket,
            5 => Self::Unused5,
            6 => Self::Unused6,
            _ => Self::Unknown,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_item_mode() {
        // v=5 = 0b101, LSB-first → byte = 0b00000101 = 0x05
        let data = [0b00000101u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemMode::read(&mut reader).unwrap(), ItemMode::Unused5);

        // v=1 = 0b001, LSB-first → byte = 0b00000001 = 0x01
        let data = [0b00000001u8];
        let mut reader = BitReader::new(&data);
        assert_eq!(ItemMode::read(&mut reader).unwrap(), ItemMode::Equipped);
    }
}