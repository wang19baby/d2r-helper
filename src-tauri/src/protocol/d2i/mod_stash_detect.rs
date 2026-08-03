//! D2I Mod Stash 检测 (仙道轮回等 mod 的 stash 文件格式探测)
//!
//! ## 背景
//! 仙道轮回 mod 改了 d2i stash 文件的 item 格式:
//! - jm_count 编码可能扩展 (mod 可能插入 mod-bit-count prefix)
//! - item code 可能是 4-char (mod 自定义扩展, 不在 ALL_ITEMS)
//! - complete header 多/少读 bits
//! - Page[31] 的 is_stackable == 2 (vanilla 只有 0/1)
//!
//! 在 vanilla 路径上 (411/411 diff=0) 这些 mod items 完全无法解析。
//! 走完整的逆向 mod protocol 不在本任务范围内。
//!
//! ## 方案
//! 提供 `detect_mod_stash()` 启发式 + `ModStashKind` 枚举,
//! 让 `parse_file` 入口在检测到 mod stash 时走降级路径
//! (`parse_file_mod_shoudao`, gated by `mod-stash-experimental` feature)。

use crate::protocol::d2i::page::Page;

/// Stash 文件类型探测结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModStashKind {
    /// vanilla D2R stash (或未知但兼容的 stash)
    Vanilla,
    /// 仙道轮回 mod stash (heuristic 命中)
    ModShoudao,
    /// 未知 stash,无法判断
    Unknown,
}

/// 启发式探测 stash 是否为 mod 类型。
///
/// 探针 (任一命中即返回 ModShoudao):
/// 1. Page[31] 的 is_stackable 字段 == 2 **且** page[0] 的 count_based 失败率 > 50%
///    (避免 ModernSharedStashSoftCoreV2 误判 — 它 page[31] data[20]=2 但 page[0-30] 全 vanilla 格式)
/// 2. 大量 count_based 失败 (jm_count - count_based_count) / jm_count > 0.50
///
/// 探针 1+1a 组合避免误判;探针 2 是软指标 (保守防漏判,阈值 50% 比 plan 中 30% 更严格)。
pub fn detect_mod_stash(
    pages: &[Page],
    page_results: &[(usize, usize)], // (jm_count, count_based_count)
) -> ModStashKind {
    // 探针 2: 大量 count_based 失败 (>50% 漏) — 这是 mod 失真的强信号
    let total_jm: usize = page_results.iter().map(|(j, _)| j).sum();
    let total_cb: usize = page_results.iter().map(|(_, c)| c).sum();
    let high_loss = if total_jm > 0 && total_jm < usize::MAX / 2 {
        let lost = total_jm.saturating_sub(total_cb);
        lost as f64 / total_jm as f64 > 0.50
    } else {
        false
    };

    // 探针 1: Page[31].is_stackable == 2 单独不充分 (ModernSharedStashSoftCoreV2
    // 的 page[31] data[20]=2 但 page[0-30] 全 vanilla 格式)
    // 探针 1a: 需要配合高 loss rate 才算 mod
    if pages.len() > 31 {
        const STACKABLE_OFFSET: usize = 20;
        if pages[31].data.len() > STACKABLE_OFFSET
            && pages[31].data[STACKABLE_OFFSET] == 2
            && high_loss
        {
            return ModStashKind::ModShoudao;
        }
    }

    if high_loss {
        return ModStashKind::ModShoudao;
    }

    ModStashKind::Vanilla
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page_with_stackable_byte(stackable_byte: u8) -> Page {
        let mut data = vec![0u8; 100];
        // magic
        data[0..4].copy_from_slice(&0xAA55AA55u32.to_le_bytes());
        // page_size
        data[16..20].copy_from_slice(&100u32.to_le_bytes());
        // is_stackable byte
        data[20] = stackable_byte;
        Page {
            index: 0,
            offset: 0,
            size: 100,
            is_stackable: stackable_byte == 1,
            data,
        }
    }

    #[test]
    fn test_detect_vanilla() {
        let pages = vec![make_page_with_stackable_byte(0)];
        let page_results = vec![(50usize, 50usize)]; // 0% lost
        assert_eq!(detect_mod_stash(&pages, &page_results), ModStashKind::Vanilla);
    }

    #[test]
    fn test_detect_mod_by_stackable_2() {
        let mut pages = vec![make_page_with_stackable_byte(0); 32];
        pages[31] = make_page_with_stackable_byte(2);
        // 80% lost → 配合 page[31]=2 触发 mod
        let page_results = vec![(100usize, 20usize); 32];
        assert_eq!(detect_mod_stash(&pages, &page_results), ModStashKind::ModShoudao);
    }

    #[test]
    fn test_detect_mod_by_high_loss() {
        let pages = vec![make_page_with_stackable_byte(0); 3];
        // 80% lost → mod
        let page_results = vec![(100usize, 20usize)];
        assert_eq!(detect_mod_stash(&pages, &page_results), ModStashKind::ModShoudao);
    }

    #[test]
    fn test_detect_low_loss_is_vanilla() {
        let pages = vec![make_page_with_stackable_byte(0); 3];
        // 10% lost → vanilla
        let page_results = vec![(100usize, 90usize)];
        assert_eq!(detect_mod_stash(&pages, &page_results), ModStashKind::Vanilla);
    }
}