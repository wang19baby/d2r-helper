//! 统计所有已解析 item 的 flags count_ones 分布。

use std::collections::HashMap;
use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
use d2r_marketplace_lib::protocol::d2i::jm_reader::{scan_next_item, ScanConfig};

fn collect_flags(path: &std::path::Path) -> Vec<u32> {
    let data = std::fs::read(path).unwrap();
    let _file = match parse_file(&data) {
        Ok(f) => return f.items.iter().map(|i| i.item.flags.raw).collect(),
        Err(_) => {
            // try legacy parser
            let pages = split_legacy_d2i_pages(&data).unwrap();
            let payload = &pages.pages[0].data[64..];
            let mut flags = Vec::new();
            let mut start = 0;
            loop {
                if start >= payload.len() * 8 { break; }
                if let Some(sr) = scan_next_item(payload, start, &ScanConfig::default()) {
                    let mut r = BitReader::new(payload);
                    r.seek(sr.position);
                    let f = r.read_u32(32);
                    flags.push(f);
                    start = sr.position + 8;
                } else { break; }
            }
            return flags;
        }
    };
}

fn print_stats(flags_list: &[u32], label: &str) {
    let total = flags_list.len() as f64;
    let mut buckets: HashMap<u32, usize> = HashMap::new();
    for &f in flags_list {
        let ones = f.count_ones();
        *buckets.entry(ones).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = buckets.into_iter().collect();
    sorted.sort_by_key(|&(k, _)| k);

    println!("\n=== {} ({} items) ===", label, flags_list.len());
    for (ones, cnt) in &sorted {
        println!("  ones={:2}: {:4} ({:5.2}%)", ones, cnt, (*cnt as f64 / total) * 100.0);
    }

    let gt6 = flags_list.iter().filter(|&&f| f.count_ones() > 6).count();
    let gt4 = flags_list.iter().filter(|&&f| f.count_ones() > 4).count();
    let gt3 = flags_list.iter().filter(|&&f| f.count_ones() > 3).count();
    println!("  >3: {} | >4: {} | >6: {}", gt3, gt4, gt6);

    let high: Vec<u32> = flags_list.iter().filter(|&&f| f.count_ones() > 4).copied().collect();
    if !high.is_empty() {
        println!("  ones>4 items:");
        for f in high {
            println!("    ones={} flags=0x{:08X}", f.count_ones(), f);
        }
    }
}

#[test]
fn flags_count_ones_stats_all_fixtures() {
    let fixtures = [
        ("ModernSharedStashSoftCoreV2.d2i", "stash"),
        ("zmb_only.d2i", "zmb"),
        ("user_stash.d2i", "user"),
    ];

    let mut all_flags: Vec<u32> = Vec::new();

    for (name, label) in fixtures {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests").join("fixtures").join(name);
        if path.exists() {
            let flags = collect_flags(&path);
            println!("{}: {} items", name, flags.len());
            print_stats(&flags, label);
            all_flags.extend(flags);
        } else {
            println!("SKIP (not found): {}", name);
        }
    }

    print_stats(&all_flags, "ALL FIXTURES COMBINED");

    // unique flags
    let mut unique: HashMap<u32, u32> = HashMap::new();
    for &f in &all_flags {
        *unique.entry(f).or_insert(0) += 1;
    }
    let mut unique_sorted: Vec<_> = unique.into_iter()
        .map(|(f, cnt)| (f.count_ones(), f, cnt))
        .collect();
    unique_sorted.sort_by_key(|&(o, _, _)| o);
    println!("\n=== unique flags ===");
    for (ones, f, cnt) in unique_sorted {
        println!("  ones={} 0x{:08X} x{}", ones, f, cnt);
    }
}
