use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum number of pages in a legacy .d2i file
/// Set high (50) to support mods with extended stash tabs
const MAX_PAGES: usize = 50;

/// Magic number identifying a legacy D2I page
const D2I_PAGE_MAGIC: u32 = 0xAA55AA55;

/// A parsed D2I page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub index: usize,
    pub offset: usize,
    pub size: usize,
    pub is_stackable: bool,
    pub data: Vec<u8>,
}

/// Parsed pages from a D2I file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D2IPages {
    pub pages: Vec<Page>,
    pub tail: Vec<u8>,
}

/// 64-byte header of a D2I page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D2IPageHeader {
    pub magic: u32,           // 0xAA55AA55
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub page_size: u32,
    pub is_stackable: u8,
    pub unk0: u8,
    pub unk1: u8,
    pub unk2: u8,
    pub reserved: Vec<u8>,    // 40 bytes, serialized as Vec for Serde compat
}

#[derive(Error, Debug)]
pub enum PageError {
    #[error("No stackable page found in the stash file")]
    NoStackablePage,
    #[error("Invalid page magic at offset {0}")]
    InvalidMagic(usize),
}

impl D2IPageHeader {
    pub const SIZE: usize = 64;

    /// Parse a page header from raw bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        let magic = u32::from_le_bytes(data[0..4].try_into().ok()?);
        if magic != D2I_PAGE_MAGIC {
            return None;
        }
        Some(Self {
            magic,
            unknown1: u32::from_le_bytes(data[4..8].try_into().ok()?),
            unknown2: u32::from_le_bytes(data[8..12].try_into().ok()?),
            unknown3: u32::from_le_bytes(data[12..16].try_into().ok()?),
            page_size: u32::from_le_bytes(data[16..20].try_into().ok()?),
            is_stackable: data[20],
            unk0: data[21],
            unk1: data[22],
            unk2: data[23],
            reserved: data[24..64].to_vec(),
        })
    }

    /// Serialize header back to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.extend_from_slice(&self.unknown1.to_le_bytes());
        bytes.extend_from_slice(&self.unknown2.to_le_bytes());
        bytes.extend_from_slice(&self.unknown3.to_le_bytes());
        bytes.extend_from_slice(&self.page_size.to_le_bytes());
        bytes.push(self.is_stackable);
        bytes.push(self.unk0);
        bytes.push(self.unk1);
        bytes.push(self.unk2);
        // Pad reserved to 40 bytes
        let reserved_padded: Vec<u8> = self.reserved.iter().copied().chain(std::iter::repeat(0u8)).take(40).collect();
        bytes.extend_from_slice(&reserved_padded);
        bytes
    }
}

/// Split a raw D2I buffer into its pages and tail data
pub fn split_legacy_d2i_pages(buffer: &[u8]) -> Result<D2IPages, PageError> {
    let mut pages = Vec::new();
    let mut offset = 0;

    for i in 0..MAX_PAGES {
        if offset + D2IPageHeader::SIZE > buffer.len() {
            break;
        }

        let header = D2IPageHeader::from_bytes(&buffer[offset..]);
        if header.is_none() {
            break;
        }
        let header = header.unwrap();

        let page_size = header.page_size as usize;
        if page_size == 0 || offset + page_size > buffer.len() {
            break;
        }

        pages.push(Page {
            index: i,
            offset,
            size: page_size,
            is_stackable: header.is_stackable == 1,
            data: buffer[offset..offset + page_size].to_vec(),
        });

        offset += page_size;
    }

    let tail = buffer[offset..].to_vec();

    Ok(D2IPages { pages, tail })
}

/// Find the stackable page from parsed pages
pub fn find_stackable_page(pages: &[Page]) -> Option<&Page> {
    pages.iter().find(|p| p.is_stackable)
}

/// Find the stackable page mutably
pub fn find_stackable_page_mut(pages: &mut [Page]) -> Option<&mut Page> {
    pages.iter_mut().find(|p| p.is_stackable)
}

/// Reassemble pages and tail back into a complete D2I buffer
pub fn reassemble_d2i(pages: &[Page], tail: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for page in pages {
        buffer.extend_from_slice(&page.data);
    }
    buffer.extend_from_slice(tail);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = D2IPageHeader {
            magic: 0xAA55AA55,
            unknown1: 0,
            unknown2: 0,
            unknown3: 0,
            page_size: 64 + 100,
            is_stackable: 1,
            unk0: 0,
            unk1: 0,
            unk2: 0,
            reserved: vec![0u8; 40],
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 64);

        let parsed = D2IPageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.magic, 0xAA55AA55);
        assert_eq!(parsed.page_size, 64 + 100);
        assert_eq!(parsed.is_stackable, 1);
    }
}
