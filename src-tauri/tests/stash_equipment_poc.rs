use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
use std::path::Path;

/// 用生产解析器分析 zmb
/// fixture 可能不随仓库分发（用户本地存档）——缺失时 SKIP。
fn fixture_path(name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures").join(name);
    if p.exists() {
        Some(p)
    } else {
        eprintln!("SKIP: fixture {} 未随仓库分发（本地存档）, 跳过测试", name);
        None
    }
}

#[test]
fn test_poc_zmb_production() {
    let fixture = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let pages = split_legacy_d2i_pages(&data).expect("Failed to parse pages");
    let page = &pages.pages[0];

    let items = d2r_marketplace_lib::protocol::d2i::legacy::item::read_stash_items_from_page(page)
        .expect("parse failed");

    // Item 0 = zmb
    let zmb = &items[0];
    let bo = zmb.raw_bit_offset / 8;
    let bl = (zmb.raw_bit_offset + zmb.raw_bit_length).div_ceil(8) - bo;

    eprintln!("zmb: offset={}B size={}B type='{}' simple={} amount={}",
        bo, bl, zmb.item_type.trim(), zmb.simple_item, zmb.amount);

    // Show first 5 items
    for (i, it) in items.iter().enumerate().take(5) {
        let bo2 = it.raw_bit_offset / 8;
        let bl2 = (it.raw_bit_offset + it.raw_bit_length).div_ceil(8) - bo2;
        eprintln!("  [{i}] offset={bo2}B size={bl2}B type='{}' simple={} amount={} q={:?}",
            it.item_type.trim(), it.simple_item, it.amount, it.quality);
    }
    eprintln!("  ... total {} items on page 0", items.len());
}
