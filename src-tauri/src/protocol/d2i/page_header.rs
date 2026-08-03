//! D2I page 顶层 header（64 字节）。
//!
//! 格式：page header magic `0xAA55AA55`（小端），3 个 unknown u32，page_size u32，
//! 然后 is_stackable u8 + 3 个 unk u8 + 40 字节 reserved。

use crate::core::ParseResult;

/// D2I 顶层 page header magic（小端 0xAA55AA55）。
pub const D2I_PAGE_MAGIC: u32 = 0xAA55AA55;

/// Header 大小固定 64 字节。
pub const D2I_PAGE_HEADER_SIZE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeader {
    pub magic: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub page_size: u32,
    pub is_stackable: u8,
    pub unk0: u8,
    pub unk1: u8,
    pub unk2: u8,
    pub reserved: [u8; 40],
}

impl PageHeader {
    /// 从原始字节解析 header。
    pub fn from_bytes(data: &[u8]) -> ParseResult<Self> {
        if data.len() < D2I_PAGE_HEADER_SIZE {
            return Err(crate::core::ParseError::Truncated(data.len()));
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != D2I_PAGE_MAGIC {
            return Err(crate::core::ParseError::PageMagic(magic));
        }
        let mut reserved = [0u8; 40];
        reserved.copy_from_slice(&data[24..64]);

        Ok(Self {
            magic,
            unknown1: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            unknown2: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            unknown3: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            page_size: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            is_stackable: data[20],
            unk0: data[21],
            unk1: data[22],
            unk2: data[23],
            reserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_header() {
        // 64-byte fixture
        let mut data = [0u8; 64];
        // magic 0xAA55AA55 → bytes 55 AA 55 AA (little-endian)
        data[0..4].copy_from_slice(&D2I_PAGE_MAGIC.to_le_bytes());
        data[16..20].copy_from_slice(&164u32.to_le_bytes()); // page_size = 164
        data[20] = 1; // is_stackable

        let h = PageHeader::from_bytes(&data).unwrap();
        assert_eq!(h.magic, 0xAA55AA55);
        assert_eq!(h.page_size, 164);
        assert_eq!(h.is_stackable, 1);
    }

    #[test]
    fn test_parse_invalid_magic() {
        let data = [0u8; 64];
        let err = PageHeader::from_bytes(&data).unwrap_err();
        match err {
            crate::core::ParseError::PageMagic(m) => assert_eq!(m, 0),
            _ => panic!("expected PageMagic error"),
        }
    }

    #[test]
    fn test_parse_truncated() {
        let data = [0u8; 32];
        let err = PageHeader::from_bytes(&data).unwrap_err();
        assert!(matches!(err, crate::core::ParseError::Truncated(_)));
    }
}