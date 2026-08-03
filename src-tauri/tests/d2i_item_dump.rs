//! 完整按 page 整理的 d2i 解析清单
//!
//! 用法: cargo test --test d2i_item_dump -- --nocapture

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn dump_items_by_page() {
    if !std::path::Path::new(REAL).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL);
        return;
    }
    let bytes = std::fs::read(REAL).expect("read d2i");
    let file = parse_file(&bytes).expect("parse_file");

    println!("\n================ D2I 解析结果 ================");
    println!("文件: ModernSharedStashSoftCoreV2.d2i");
    println!("大小: {} bytes", bytes.len());
    println!("pages: {}, items: {}\n", file.pages.len(), file.items.len());

    let total_pages = file.pages.len();
    for page_idx in 0..total_pages {
        let page = &file.pages[page_idx];
        let pd = page.data.as_slice();
        let jm = if pd.len() >= 4 && &pd[0x40..0x42] == b"JM" {
            u16::from_le_bytes([pd[0x42], pd[0x43]]) as usize
        } else { 0 };
        let page_items: Vec<_> = file.items.iter()
            .filter(|it| it.page_index == page_idx)
            .collect();

        if page_items.is_empty() && jm == 0 {
            continue;
        }

        let kind = if page.is_stackable { "stackable" } else { "equipment" };
        println!("---- Page[{:>2}]  {}  size={}B  jm={}  parsed={} ----",
            page_idx, kind, page.size, jm, page_items.len());

        for (i, p) in page_items.iter().enumerate() {
            let code = p.item.code.trim();
            let q = format!("{:?}", p.item.quality);
            let stat_count: usize = p.item.stat_lists.iter()
                .map(|sl| sl.stats.len()).sum();
            println!(
                "  {:>3}. code={}  x={:<2} y={:<2}  q={:<11}  ilvl={:>3}  sockets={}  stats={}",
                i, code, p.item.x, p.item.y, q, p.item.item_level, p.item.num_sockets, stat_count
            );
        }
        println!();
    }

    // 汇总
    println!("================ Summary ================");
    use std::collections::HashMap;
    let mut by_kind: HashMap<String, usize> = HashMap::new();
    let mut by_quality: HashMap<String, usize> = HashMap::new();
    for it in &file.items {
        let cs = it.item.code.trim();
        let k = if cs.starts_with("r0") || cs.starts_with("r1") || cs.starts_with("r2") || cs.starts_with("r3") {
            "rune".to_string()
        } else if cs == "gcv" || cs == "gfv" || cs == "gsv" || cs == "gpv" || cs == "gzv" {
            "gem".to_string()
        } else if cs.starts_with("amu") {
            "amulet".to_string()
        } else if cs.starts_with("rin") {
            "ring".to_string()
        } else if cs.starts_with("jew") {
            "jewel".to_string()
        } else {
            format!("other({})", cs)
        };
        *by_kind.entry(k).or_insert(0) += 1;
        let q = format!("{:?}", it.item.quality);
        *by_quality.entry(q).or_insert(0) += 1;
    }
    println!("\n[by item_kind]");
    let mut v: Vec<_> = by_kind.iter().collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (k, c) in v {
        println!("  {:>16} : {}", k, c);
    }
    println!("\n[by quality]");
    let mut v: Vec<_> = by_quality.iter().collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (k, c) in v {
        println!("  {:>16} : {}", k, c);
    }
}
