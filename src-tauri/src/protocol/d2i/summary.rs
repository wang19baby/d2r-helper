//! d2i 解析摘要（轻量级，不展开 stat_list）。
//!
//! 用于 commands::stash 的快速概览路径，
//! 避免完整 stat_list 解析的开销。

use crate::protocol::d2i::parser::D2IFile;
use serde::{Deserialize, Serialize};

/// 单个 item 摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub page_index: usize,
    pub code: String,
    pub quality: u8,
    pub x: u8,
    pub y: u8,
    pub amount: u32,
    pub identified: bool,
    pub socketed: bool,
    pub ethereal: bool,
    pub is_runeword: bool,
}

/// 完整 stash 摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashSummary {
    pub pages: usize,
    pub stackable_pages: usize,
    pub total_items: usize,
    pub items: Vec<ItemSummary>,
}

/// 从 D2IFile 生成轻量摘要。
pub fn summarize(file: &D2IFile) -> StashSummary {
    let items = file
        .items
        .iter()
        .map(|p| ItemSummary {
            page_index: p.page_index,
            code: p.item.code.clone(),
            quality: p.item.quality.as_u8(),
            x: p.item.x,
            y: p.item.y,
            amount: p.item.amount,
            identified: p.item.flags.identified(),
            socketed: p.item.flags.socketed(),
            ethereal: p.item.flags.ethereal(),
            is_runeword: p.item.flags.is_runeword(),
        })
        .collect();

    StashSummary {
        pages: file.pages.len(),
        stackable_pages: file.pages.iter().filter(|p| p.is_stackable).count(),
        total_items: file.items.len(),
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_empty() {
        let file = D2IFile {
            pages: vec![],
            items: vec![],
            tail: vec![],
        };
        let s = summarize(&file);
        assert_eq!(s.pages, 0);
        assert_eq!(s.total_items, 0);
    }
}