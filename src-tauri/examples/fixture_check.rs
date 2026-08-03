// Fixture 验证: 用真实 d2s 文件检查修复后 ItemStat 是否正确拆分
use d2r_marketplace_lib::protocol::d2s::items::read_standard_items;
use std::path::Path;

fn main() {
    let fixtures = [
        "xieedi.d2s",
        "happy_manman.d2s",
        "librarian.d2s",
        "EchoingStrike.d2s",
        "standard_test_warlock_tc03.d2s",
    ];
    for name in &fixtures {
        let path = format!("tests/fixtures/{}", name);
        if !Path::new(&path).exists() { continue; }
        let data = std::fs::read(&path).expect("read");
        println!("\n=== {} ({} bytes) ===", name, data.len());
        match read_standard_items(&data) {
            Ok(items) => {
                println!("  parsed {} items", items.len());
                let mut found_skill_stats = 0;
                for item in &items {
                    for sl in &item.item.stat_lists {
                        for s in &sl.stats {
                            if s.skill_tab.is_some() || s.skill_id.is_some() || s.max_charges.is_some() {
                                found_skill_stats += 1;
                                println!("  ★ code={:?} id={} tab={:?} level={:?} skill_id={:?} max_ch={:?} value={}",
                                    item.item.code, s.id, s.skill_tab, s.skill_level, s.skill_id, s.max_charges, s.value);
                            }
                        }
                    }
                }
                println!("  found {} skill-related stat entries", found_skill_stats);
            }
            Err(e) => println!("  ERR: {:?}", e),
        }
    }
}
