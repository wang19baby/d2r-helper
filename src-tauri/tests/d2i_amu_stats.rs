//! Page[0] amu (Rare) 抗性 stat 分析

use d2r_marketplace_lib::protocol::d2i::page::split_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_page0_amu_stats() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let (pages, _) = split_pages(&bytes).expect("split");
    let _ = pages.first().expect("page 0");

    let file = parse_file(&bytes).expect("parse");

    println!("=== Page[0] 所有 amu + xmg + Set 质量 ===");
    for p in file.items.iter().filter(|p| p.page_index == 0 && (p.item.code == "amu" || p.item.code == "xmg")) {
        println!(
            "  bit={} q={:?} x={} y={} stat_lists={} stat_count={}",
            p.raw_bit_offset, p.item.quality, p.item.x, p.item.y,
            p.item.stat_lists.len(),
            p.item.stat_lists.iter().map(|sl| sl.stats.len()).sum::<usize>()
        );
    }
    println!("\n=== Page[0] Set 质量全部 ===");
    for p in file.items.iter().filter(|p| p.page_index == 0) {
        let q = format!("{:?}", p.item.quality);
        if q == "Set" {
            println!("  bit={} code='{}' stat_lists={} stat_count={}",
                p.raw_bit_offset, p.item.code, p.item.stat_lists.len(),
                p.item.stat_lists.iter().map(|sl| sl.stats.len()).sum::<usize>());
            if p.item.code == "xmg" {
                println!("    ↓ DETAIL of xmg:");
                for (i, sl) in p.item.stat_lists.iter().enumerate() {
                    println!("    sl[{}] stats:", i);
                    for s in &sl.stats {
                        println!("      id={} value={}", s.id, s.value);
                    }
                }
            }
        }
    }
}
