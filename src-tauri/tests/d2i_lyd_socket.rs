//! 看 lyd 是不是有 socket,以及 r06 是不是 socketed sub

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_lyd_socket() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let file = parse_file(&bytes).expect("parse");

    let lyd: Vec<_> = file.items.iter()
        .filter(|it| it.page_index == 5 && it.item.code.trim() == "lyd")
        .collect();
    println!("lyd count: {}", lyd.len());
    for p in &lyd {
        println!(
            "  bit_off={} bit_len={} num_sockets={} socketed_items={} amount={} code='{}'",
            p.raw_bit_offset, p.raw_bit_length,
            p.item.num_sockets,
            p.item.socketed_items.len(),
            p.item.amount,
            p.item.code
        );
        for s in &p.item.socketed_items {
            println!("    socket: code='{}' amount={}", s.code, s.amount);
        }
    }

    // 列出所有 lyd amount 单独值
    let lyd_amounts: Vec<u32> = lyd.iter().map(|p| p.item.amount).collect();
    println!("lyd amounts: {:?}", lyd_amounts);

    // 所有 Page[5] item 的 amount 累加,看 lyd
    let mut by_amount: std::collections::BTreeMap<&str, (usize, u32)> = std::collections::BTreeMap::new();
    for pi in file.items.iter().filter(|it| it.page_index == 5) {
        let entry = by_amount.entry(pi.item.code.as_str()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += pi.item.amount;
    }
    println!("\nPage[5] by (code: count, total_amount):");
    for (code, (cnt, total)) in &by_amount {
        println!("  '{}': count={} total={}", code, cnt, total);
    }

    // 检查 r06 在 任何 socketed_items 里
    let r06_sockets: usize = file.items.iter()
        .filter(|it| it.page_index == 5)
        .map(|p| p.item.socketed_items.iter()
            .filter(|s| s.code.trim() == "r06")
            .count())
        .sum();
    println!("\nr06 in socketed_items: {}", r06_sockets);
}
