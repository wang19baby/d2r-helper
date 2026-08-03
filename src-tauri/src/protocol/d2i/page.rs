//! D2I page 容器（header + item 数据）。

use crate::core::ParseResult;
use crate::protocol::d2i::page_header::{PageHeader, D2I_PAGE_HEADER_SIZE};

/// 一个 D2I page。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub index: usize,
    pub offset: usize,
    pub size: usize,
    pub is_stackable: bool,
    /// 完整 page 数据（含 64-byte header）
    pub data: Vec<u8>,
}

impl Page {
    /// 仅取 item 数据部分（跳过 64-byte header）。
    pub fn item_bytes(&self) -> &[u8] {
        if self.data.len() > D2I_PAGE_HEADER_SIZE {
            &self.data[D2I_PAGE_HEADER_SIZE..]
        } else {
            &[]
        }
    }
}

/// 从 raw buffer 拆出 pages（保留 tail 用于回写）。
pub fn split_pages(buffer: &[u8]) -> ParseResult<(Vec<Page>, Vec<u8>)> {
    let mut pages = Vec::new();
    let mut offset = 0;
    const MAX_PAGES: usize = 50;

    for i in 0..MAX_PAGES {
        if offset + D2I_PAGE_HEADER_SIZE > buffer.len() {
            break;
        }
        // 尝试解析 header；magic 不匹配则停止
        let header = match PageHeader::from_bytes(&buffer[offset..]) {
            Ok(h) => h,
            Err(_) => break,
        };

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
    Ok((pages, tail))
}

/// 找堆叠页（runes/gems/keys 等）。
pub fn find_stackable_page(pages: &[Page]) -> Option<&Page> {
    pages.iter().find(|p| p.is_stackable)
}

/// 把 pages + tail 重新拼回完整 d2i buffer（回写用）。
///
/// 与 `protocol::d2i::legacy::page::reassemble_d2i` 等价,但作用于新 `Page` 类型。
/// 用于仓库 deposit/withdraw 等需要修改单页后再写回的场景。
pub fn reassemble_pages(pages: &[Page], tail: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(
        pages.iter().map(|p| p.data.len()).sum::<usize>() + tail.len(),
    );
    for page in pages {
        buffer.extend_from_slice(&page.data);
    }
    buffer.extend_from_slice(tail);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page_bytes(page_size: u32, is_stackable: u8) -> Vec<u8> {
        let mut data = vec![0u8; page_size as usize];
        // magic
        let magic = 0xAA55AA55u32;
        data[0..4].copy_from_slice(&magic.to_le_bytes());
        data[16..20].copy_from_slice(&page_size.to_le_bytes());
        data[20] = is_stackable;
        data
    }

    #[test]
    fn test_split_single_page() {
        let buffer = make_page_bytes(100, 1);
        let (pages, tail) = split_pages(&buffer).unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].is_stackable);
        assert_eq!(pages[0].size, 100);
        assert_eq!(tail.len(), 0);
    }

    #[test]
    fn test_split_two_pages_with_tail() {
        let mut buffer = Vec::new();
        buffer.extend(make_page_bytes(80, 0));
        buffer.extend(make_page_bytes(120, 1));
        buffer.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // tail bytes

        let (pages, tail) = split_pages(&buffer).unwrap();
        assert_eq!(pages.len(), 2);
        assert!(!pages[0].is_stackable);
        assert!(pages[1].is_stackable);
        assert_eq!(tail, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_item_bytes_skips_header() {
        let buffer = make_page_bytes(100, 1);
        let (pages, _) = split_pages(&buffer).unwrap();
        let p = &pages[0];
        assert_eq!(p.item_bytes().len(), 100 - 64);
    }
}