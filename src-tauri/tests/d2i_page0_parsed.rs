//! Page[0] 解析流程追踪 — count-based 跑多少 + v3 fallback 跑多少
//!
//! 看 v3 fallback 是否真的续接 80 items

use d2r_marketplace_lib::protocol::d2i::page::split_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
use std::collections::HashSet;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_page0_parsed_items() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let (pages, _) = split_pages(&bytes).expect("split");
    let _ = pages.first().expect("page 0");

    let file = parse_file(&bytes).expect("parse");
    let page0_items: Vec<_> = file.items.iter()
        .filter(|p| p.page_index == 0)
        .collect();

    println!("=== Page[0] parse_file 实际解出 {} 个 item ===", page0_items.len());
    let parsed_bit_set: HashSet<usize> = page0_items.iter()
        .map(|p| p.raw_bit_offset)
        .collect();

    // 显示所有解出的 bit + 它们的 quality
    for p in &page0_items {
        println!("  bit {}: code='{}' q={:?}",
            p.raw_bit_offset, p.item.code.trim(), p.item.quality);
    }

    // 跟 full_scan 91 个 candidate 对比 (bit 已知)
    // full scan 找到的关键 bit (从 page0_full_scan 输出):
    let expected_bits: Vec<(usize, &str)> = vec![
        (32, "gth"), (280, "nea"), (672, "8rx"), (968, "gwn"),
        (1240, "rin"), (1616, "rin"), (2008, "xul"), (2264, "jew"),
        (2848, "rin"), (3096, "utp"), (4008, "7vo"), (4160, "7vo"),
        (4312, "rin"), (4672, "amu"), (5056, "7pa"), (5208, "rin"),
        (5528, "rin"), (5784, "7vo"), (5936, "rin"), (6544, "jew"),
        (6800, "rin"), (7048, "obf"), (7296, "9sm"), (7584, "7dg"),
        (7896, "jew"), (8144, "jew"), (8392, "rin"), (8600, "amu"),
        (8840, "ba5"), (9120, "ba5"), (9400, "xlb"), (9744, "xlb"),
        (10088, "rin"), (10424, "rin"), (10816, "9bl"), (11088, "8rx"),
        (11384, "hbl"), (11672, "xhg"), (12064, "amu"), (12384, "amu"),
        (12624, "rin"), (12960, "8lw"), (13264, "lsd"), (13552, "r07"),
        (13640, "r10"), (13728, "r09"), (13816, "r11"), (13904, "rin"),
        (14152, "amu"), (14368, "jew"), (14672, "amu"), (14944, "rin"),
        (15168, "2hs"), (15456, "dr1"), (15816, "r04"), (15904, "r03"),
        (15992, "ci0"), (16392, "hbw"), (16744, "r03"), (16832, "r07"),
        (16920, "r11"), (17008, "jew"), (17264, "uap"), (17576, "jew"),
        (17824, "jew"), (18072, "amu"), (18320, "hla"), (18608, "r07"),
        (18696, "r05"), (19016, "rin"), (18784, "amu"),
    ];

    let mut missing: Vec<(usize, &str)> = Vec::new();
    for (bit, code) in &expected_bits {
        if !parsed_bit_set.contains(bit) {
            missing.push((*bit, *code));
        }
    }
    println!("\n缺失 item ({} 个):", missing.len());
    for (bit, code) in &missing {
        println!("  bit {}: code='{}'", bit, code);
    }
}
