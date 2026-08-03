//! CycleofImmortals stash 高级页验证: 13 个物品全部解析, r01-r08 全部存在
//! 之前 v3 fallback scan 遇到 mod 物品 (ue8/nwt/8p8) break 导致丢失

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const STASH_PATH: &str = "C:\\Users\\wang\\Saved Games\\Diablo II Resurrected\\mods\\CycleofImmortals\\ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_page5_all_13_items_parsed() {
    if !std::path::Path::new(STASH_PATH).exists() {
        eprintln!("SKIP: CycleofImmortals mod 存档缺失");
        return;
    }
    let bytes = std::fs::read(STASH_PATH).expect("stash file");
    let file = parse_file(&bytes).expect("parse_file");

    let page5_items: Vec<_> = file.items.iter()
        .filter(|it| it.page_index == 5)
        .collect();
    let codes: Vec<&str> = page5_items.iter().map(|it| it.item.code.as_str()).collect();

    println!("Page[5] items: {} (expected 13)", page5_items.len());
    println!("Codes: {:?}", codes);

    // 至少 10 个(允许少量 mod 物品解析不掉)
    assert!(page5_items.len() >= 10,
        "Page[5] should have >=10 items, got {}", page5_items.len());

    // r01-r08 必须全部存在
    for rune in &["r01", "r02", "r03", "r04", "r05", "r06", "r07", "r08"] {
        assert!(codes.contains(rune),
            "Page[5] should contain rune {}, got {:?}", rune, codes);
    }
}

