//! 物品位级布局 / 构建显示函数
//!
//! 注: `dump_item_bits` 及 bit reader helpers (read_bits_as / read_u32_at)
//! 拆到 `dump_item.rs`。这个文件保留 orchestrator + show_items_bits_list
//! + format_item_line。

use std::path::Path;
use std::time::Duration;

use d2r_marketplace_lib::protocol::d2s::parse_file as parse_d2s;

use crate::display;
use crate::dump_item::dump_item_bits;
use crate::helpers;

// ═══════════════════════════════════════════════
// Items list for --bits (no index)
// ═══════════════════════════════════════════════

fn show_items_bits_list(path: &Path) -> String {
    let data = std::fs::read(path).unwrap_or_default();
    use d2r_marketplace_lib::protocol::d2s::items::{read_standard_items, read_merc_items};
    use d2r_marketplace_lib::protocol::d2s::parser::marker_offsets;

    let m = marker_offsets(&data);
    let jm_offset = match m.first_jm { Some(o) => o, None => return "No JM section found.".to_string() };
    if jm_offset + 4 > data.len() { return "JM section too short.".to_string(); }
    let item_count = u16::from_le_bytes([data[jm_offset + 2], data[jm_offset + 3]]) as usize;

    let items = read_standard_items(&data).unwrap_or_default();
    let merc_items = read_merc_items(&data);
    let merc_count = merc_items.len();

    // Function to format one item line
    let mut lines = Vec::new();
    lines.push("# Flags 含义: I=已辨识  So=已镶嵌  N=新建  S=起始物品  C=压缩存储  E=无形  P=已打孔  W=符文之语  bit23=3(未知)".to_string());
    lines.push(format!("{:>6} {:1} {:>4}  {:>4}  ver {:>2} {:>2}  {:>6}  {:10}  {:28}  {:>8}  stats",
        "[idx]", "", "off", "len", "x", "y", "pos", "code", "name", "flags"));
    let mut idx = 0;

    // Format main items
    for it in &items {
        let (pos_name, final_disp) = format_item_line(it, false);
        let kind = if it.item.flags.raw & (1 << 16) != 0 { 'E' } else if (it.item.flags.raw >> 21) & 1 == 1 { 'C' } else { 'F' };
        let sc: usize = it.item.stat_lists.iter().map(|sl| sl.stats.len()).sum();
        let length = it.raw_bit_length.div_ceil(8);
        let fl = it.item.flags.raw;
        let mut flag_parts: Vec<&str> = Vec::new();
        if fl & (1<<4) != 0 { flag_parts.push("I"); }
        if fl & (1<<11) != 0 { flag_parts.push("So"); }
        if fl & (1<<13) != 0 { flag_parts.push("N"); }
        if fl & (1<<17) != 0 { flag_parts.push("S"); }
        if fl & (1<<21) != 0 { flag_parts.push("C"); }
        if fl & (1<<22) != 0 { flag_parts.push("E"); }
        if fl & (1<<23) != 0 { flag_parts.push("3"); }
        if fl & (1<<24) != 0 { flag_parts.push("P"); }
        if fl & (1<<26) != 0 { flag_parts.push("W"); }
        let flag_str = if flag_parts.is_empty() { "-".to_string() } else { flag_parts.join("+") };
        lines.push(format!("  [{:2}] {} off={:3}B  len={:3}B  ver={}  x={:2} y={:2}  {:>6}  {:10}  {:28}  {:>8}  stats={}",
            idx, kind, it.raw_bit_offset / 8, length,
            it.item.version_raw, it.item.x, it.item.y, pos_name,
            if it.item.code.is_empty() { "(空)".to_string() } else { it.item.code.clone() },
            final_disp, flag_str, sc));
        idx += 1;
    }

    // Format merc items
    for it in &merc_items {
        let (pos_name, final_disp) = format_item_line(it, true);
        let kind = if it.item.flags.raw & (1 << 16) != 0 { 'E' } else if (it.item.flags.raw >> 21) & 1 == 1 { 'C' } else { 'F' };
        let sc: usize = it.item.stat_lists.iter().map(|sl| sl.stats.len()).sum();
        let length = it.raw_bit_length.div_ceil(8);
        let fl = it.item.flags.raw;
        let mut flag_parts: Vec<&str> = Vec::new();
        if fl & (1<<4) != 0 { flag_parts.push("I"); }
        if fl & (1<<11) != 0 { flag_parts.push("So"); }
        if fl & (1<<13) != 0 { flag_parts.push("N"); }
        if fl & (1<<17) != 0 { flag_parts.push("S"); }
        if fl & (1<<21) != 0 { flag_parts.push("C"); }
        if fl & (1<<22) != 0 { flag_parts.push("E"); }
        if fl & (1<<23) != 0 { flag_parts.push("3"); }
        if fl & (1<<24) != 0 { flag_parts.push("P"); }
        if fl & (1<<26) != 0 { flag_parts.push("W"); }
        let flag_str = if flag_parts.is_empty() { "-".to_string() } else { flag_parts.join("+") };
        lines.push(format!("  [{:2}] {} off={:3}B  len={:3}B  ver={}  x={:2} y={:2}  {:>6}  {:10}  {:28}  {:>8}  stats={}",
            idx, kind, it.raw_bit_offset / 8, length,
            it.item.version_raw, it.item.x, it.item.y, pos_name,
            if it.item.code.is_empty() { "(空)".to_string() } else { it.item.code.clone() },
            final_disp, flag_str, sc));
        idx += 1;
    }

    let switch_count = items.len().saturating_sub(item_count);
    let mut parts = vec![format!("JM count: {}", item_count)];
    if switch_count > 0 { parts.push(format!("切换装备 {}", switch_count)); }
    if merc_count > 0 { parts.push(format!("佣兵 {}", merc_count)); }
    lines.push(format!("已找到: {}, {}", items.len() + merc_count, parts.join(" + ")));
    lines.join("\n")
}

/// Format one item's position name and display string.
fn format_item_line(it: &d2r_marketplace_lib::protocol::d2i::parser::ParsedItem, is_merc: bool) -> (String, String) {
    use d2r_marketplace_lib::protocol::common::{ItemLocation, ItemPage};
    let lv = if it.item.item_level > 0 { format!("({})", it.item.item_level) } else { String::new() };
    let name = it.item.code.clone();
    let mut final_disp = format!("{}{}", name, lv);

    let pos_name = if is_merc {

        match it.item.location {
            ItemLocation::Head => "佣兵[头上]",
            ItemLocation::Neck => "佣兵[项链]",
            ItemLocation::Torso => "佣兵[衣服]",
            ItemLocation::RightHand => "佣兵[武器]",
            ItemLocation::LeftHand => "佣兵[盾牌]",
            ItemLocation::Waist => "佣兵[腰带]",
            ItemLocation::Hands => "佣兵[手套]",
            _ => "佣兵",
        }.to_string()
    } else {
        match it.item.page {
            Some(ItemPage::Equipped) => {
                if it.item.mode as u8 == 2 { "腰带".to_string() } else {

                    match it.item.location {
                        ItemLocation::Head => "装备[头上]",
                        ItemLocation::Neck => "装备[项链]",
                        ItemLocation::Torso => "装备[衣服]",
                        ItemLocation::RightHand => "装备[武器]",
                        ItemLocation::LeftHand => "装备[盾牌]",
                        ItemLocation::RightFinger | ItemLocation::LeftFinger => "装备[戒指]",
                        ItemLocation::Waist => "腰带",
                        ItemLocation::Feet => "装备[鞋子]",
                        ItemLocation::Hands => "装备[手套]",
                        ItemLocation::Trinket1 => "装备[副武器]",
                        ItemLocation::Trinket2 => "装备[副盾牌]",
                        _ => "身上",
                    }.to_string()
                }
            }
            Some(ItemPage::Backpack) => "背包".to_string(),
            Some(ItemPage::MyStash) | Some(ItemPage::Mod(5)) => "储藏箱".to_string(),
            Some(ItemPage::Mod(4)) => "盒子".to_string(),
            _ => String::new(),
        }
    };

    // Belt position
    let result_pos = if pos_name == "腰带" || pos_name.starts_with("佣兵") && it.item.location == ItemLocation::Waist {
        let belt_row = (it.item.x / 4) + 1;
        let belt_col = (it.item.x % 4) + 1;
        final_disp = format!("{}{}[排{},位{}]", name, lv, belt_row, belt_col);
        if pos_name == "腰带" { "腰带".to_string() } else { pos_name }
    } else {
        pos_name
    };

    (result_pos, final_disp)
}

// ═══════════════════════════════════════════════
// Run once
// ═══════════════════════════════════════════════

pub(crate) fn run_once(path: &Path, json_mode: bool, detail: bool, bits: Option<i32>) {
    let data = std::fs::read(path).unwrap_or_else(|e| {
        eprintln!("错误: 无法读取文件: {}", e);
        std::process::exit(1);
    });

    match bits {
        Some(-1) => {
            // Items list
            println!("{}", show_items_bits_list(path));
            return;
        }
        Some(n) if n >= 0 => {
            // Specific item bit dump
            use d2r_marketplace_lib::protocol::d2s::items::{read_standard_items, read_merc_items};
            use d2r_marketplace_lib::protocol::d2s::parser::marker_offsets;
            let m = marker_offsets(&data);
            let search_end = m.jf.or(m.kf).unwrap_or(data.len()).min(data.len());
            let items = read_standard_items(&data).unwrap_or_default();
            let merc_items = read_merc_items(&data);
            let n = n as usize;
            if n < items.len() {
                // Standard item
                let jm_offset = match m.first_jm { Some(o) => o, _ => { eprintln!("No JM section"); return; } };
                let payload = &data[jm_offset + 4..search_end];
                let it = &items[n];
                println!("Item[{}] {:?} - bit dump:", n, it.item.code);
                println!("{}", dump_item_bits(payload, it.raw_bit_offset));
                return;
            }
            let merc_idx = n - items.len();
            if merc_idx < merc_items.len() {
                // Mercenary item
                let jm_offset = match m.merc_jm { Some(o) => o, _ => { eprintln!("No merc JM section"); return; } };
                let payload = &data[jm_offset + 4..];
                let it = &merc_items[merc_idx];
                println!("Item[{}] (佣兵) {:?} - bit dump:", n, it.item.code);
                println!("{}", dump_item_bits(payload, it.raw_bit_offset));
                return;
            }
            eprintln!("错误: 物品索引 {} 超出范围 (共 {} 个标准 + {} 佣兵)", n, items.len(), merc_items.len());
            return;
        }
        _ => {}
    }

    let info = match parse_d2s(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("解析失败: {}", e);
            std::process::exit(1);
        }
    };
    display::render(&info, path, json_mode, detail);
}

// ═══════════════════════════════════════════════
// Watch loop
// ═══════════════════════════════════════════════

pub(crate) fn watch_loop(path: &Path, json_mode: bool, detail: bool, interval_secs: f64) {
    let mut last_hash = String::new();
    let interval_ms = std::cmp::max((interval_secs * 1000.0) as u64, 500);
    loop {
        if !path.exists() {
            eprintln!("[监控] 文件消失: {}", path.display());
            std::thread::sleep(Duration::from_millis(interval_ms));
            continue;
        }
        let current = helpers::file_hash(path);
        if current != last_hash {
            last_hash = current;
            let ts = chrono::Local::now().format("%H:%M:%S");
            println!("\n── [{}] {} 已更新 ──", ts, path.file_name().unwrap_or_default().to_string_lossy());
            run_once(path, json_mode, detail, None);
        }
        std::thread::sleep(Duration::from_millis(interval_ms));
    }
}
