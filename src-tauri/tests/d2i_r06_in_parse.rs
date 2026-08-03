//! 看 r06 在 Page[5] 的 item.amount

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_r06_in_parse() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let file = parse_file(&bytes).expect("parse");

    // 看 Page[5] 总 item 数
    let page5_count = file.items.iter().filter(|it| it.page_index == 5).count();
    println!("Page[5] total items: {}", page5_count);

    // 看 Page[5] bit 1048 附近的所有 item
    let r06: Vec<_> = file.items.iter()
        .filter(|it| it.page_index == 5 && it.raw_bit_offset >= 900 && it.raw_bit_offset <= 1200)
        .collect();
    println!("\nPage[5] items with bit_off 900..1200:");
    for p in &r06 {
        println!("  bit_off={} code='{}' amount={}",
            p.raw_bit_offset, p.item.code.trim(), p.item.amount);
    }

    // All r06
    let all_r06: Vec<_> = file.items.iter()
        .filter(|it| it.page_index == 5 && it.item.code.trim() == "r06")
        .collect();
    println!("\nAll r06 in Page[5]: {}", all_r06.len());
    for p in &all_r06 {
        println!("  bit_off={} amount={}", p.raw_bit_offset, p.item.amount);
    }

    // 累加 by_code (跟 test 一致)
    use std::collections::BTreeMap;
    let mut by_code: BTreeMap<&str, u32> = BTreeMap::new();
    for pi in file.items.iter().filter(|it| it.page_index == 5) {
        *by_code.entry(pi.item.code.as_str()).or_insert(0) += pi.item.amount;
    }
    println!("\nBy code (amount 累加):");
    for (code, amt) in &by_code {
        println!("  '{}' = {}", code, amt);
    }
}
