//! Page[0] xmg (Set) 状态

use d2r_marketplace_lib::protocol::d2i::page::split_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
use d2r_marketplace_lib::protocol::common::ItemQuality;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_page0_xmg_state() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let (pages, _) = split_pages(&bytes).expect("split");
    let _ = pages.first().expect("page 0");

    let file = parse_file(&bytes).expect("parse");

    println!("=== Page[0] xmg ===");
    for p in file.items.iter().filter(|p| p.page_index == 0 && p.item.code == "xmg") {
        println!(
            "  bit={} q={:?} x={} y={} stat_lists={}",
            p.raw_bit_offset, p.item.quality, p.item.x, p.item.y, p.item.stat_lists.len()
        );
        for (i, sl) in p.item.stat_lists.iter().enumerate() {
            println!("    sl[{}] ({} stats):", i, sl.stats.len());
            for s in &sl.stats {
                println!("      id={} value={}", s.id, s.value);
            }
        }
    }

    println!("\n=== Set 质量 Page[0] 全部 ===");
    for p in file.items.iter().filter(|p| p.page_index == 0 && p.item.quality == ItemQuality::Set) {
        println!("  bit={} code='{}' stat_lists={}",
            p.raw_bit_offset, p.item.code, p.item.stat_lists.len());
    }
}
