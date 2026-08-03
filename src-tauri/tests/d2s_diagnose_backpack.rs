//! 诊断：扫描 .d2s 文件全文件找合法 Huffman 物品，定位背包/腰带数据。
//!
//! 用法: cd src-tauri && cargo test --test d2s_diagnose_backpack -- --nocapture
//!
//! 扫描策略：
//!   1. 完整 d2s parser (parse_d2s) — 看标准路径能拿什么
//!   2. detect_modified_layout — 是否走修改版路径
//!   3. byte-aligned 扫描整个文件 — 找所有合法 Huffman 4-char 物品码
//!   4. 按 offset 区间分组 — 装备区、名字后区、尾部区
//!   5. 验证 x/y 坐标范围是否合理 (0..16)


use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::protocol::d2i::parser::parse_item_stream_sequential;
use d2r_marketplace_lib::protocol::d2s::items_modified::{
    detect_modified_layout, MOD_NAME_OFFSET,
};
use d2r_marketplace_lib::protocol::d2s::items::read_standard_items;

const D2S_PATH: &str = r"C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s";

/// 所有 ASCII 字母数字字符
#[allow(dead_code)]
fn is_valid_char(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

#[test]
fn diagnose_backpack() {
    if !std::path::Path::new(D2S_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地 mod 存档）, 跳过测试", D2S_PATH);
        return;
    }
    let data = std::fs::read(D2S_PATH).expect("read d2s");
    println!("========== 开心图书馆长.d2s 诊断 ==========");
    println!("文件大小: {} bytes ({} KB)", data.len(), data.len() / 1024);

    // 1. detect_modified_layout
    let is_mod = detect_modified_layout(&data);
    println!("\n--- detect_modified_layout ---");
    println!("结果: {}", if is_mod { "是 (修改版 layout)" } else { "否 (标准 layout)" });

    // 2. 标准 d2s parser
    println!("\n--- parse_d2s (标准解析) ---");
    match d2r_marketplace_lib::protocol::d2s::parser::parse_file(&data) {
        Ok(f) => {
            println!("名称: {}", f.header.name);
            println!("class: {}", f.header.class);
            println!("level: {}", f.attributes.level.raw);
            let total = f.equipped.len() + f.belt.len() + f.backpack.len() + f.cube.len() + f.merc.len();
            println!("全 items 数: {} (equipped={} belt={} backpack={} cube={} merc={})",
                total, f.equipped.len(), f.belt.len(), f.backpack.len(), f.cube.len(), f.merc.len());
            for (i, it) in f.backpack.iter().enumerate() {
                println!("  backpack[{}] code='{}' x={} y={} amt={}", i, it.item.code.trim(), it.item.x, it.item.y, it.item.amount);
            }
            for (i, it) in f.belt.iter().enumerate() {
                println!("  belt[{}] code='{}' x={} y={} amt={}", i, it.item.code.trim(), it.item.x, it.item.y, it.item.amount);
            }
        }
        Err(e) => println!("parse_d2s 失败: {:?}", e),
    }

    // 3. read_standard_items (通过 JM section)
    println!("\n--- read_standard_items (JM section D2I parser) ---");
    match read_standard_items(&data) {
        Ok(items) => {
            println!("解析到 {} 个 item (含 socketed)", items.len());
            for (i, it) in items.iter().enumerate() {
                println!("  [{}] code='{}' x={} y={} mode={:?} loc={:?} ilvl={} amt={}",
                    i, it.item.code.trim(), it.item.x, it.item.y, it.item.mode, it.item.location,
                    it.item.item_level, it.item.amount);
            }
        }
        Err(e) => println!("read_standard_items 失败: {:?}", e),
    }


    // 5. 全文件 byte-aligned Huffman 扫描
    println!("\n--- 全文件 byte-aligned Huffman 扫描 ---");
    let valid_codes: Vec<(usize, String, usize)> = scan_all_huffman_codes(&data);
    println!("找到 {} 个合法 Huffman 物品码\n", valid_codes.len());

    // 按区间分组
    let mod_end = MOD_NAME_OFFSET; // 0x12B
    println!("--- 区间分组 ---");
    println!("装备区 (0x00..0x{:X}, 修改版 12B stride):", MOD_NAME_OFFSET);
    let equip_zone: Vec<_> = valid_codes.iter().filter(|(off, _, _)| *off < mod_end).collect();
    for (off, code, bits) in &equip_zone {
        println!("  [0x{:04X}] '{}' ({} bits)", off, code, bits);
    }
    println!("  小计: {} 个", equip_zone.len());

    println!("\n名字后区 (0x{:X}..0x200):", MOD_NAME_OFFSET);
    let mid_zone: Vec<_> = valid_codes.iter().filter(|(off, _, _)| *off >= mod_end && *off < 0x200).collect();
    for (off, code, bits) in &mid_zone {
        println!("  [0x{:04X}] '{}' ({} bits)", off, code, bits);
    }
    println!("  小计: {} 个", mid_zone.len());

    println!("\n文件后半区 (0x200..0x{:X}):", data.len());
    let tail_zone: Vec<_> = valid_codes.iter().filter(|(off, _, _)| *off >= 0x200).collect();
    for (off, code, bits) in &tail_zone {
        println!("  [0x{:04X}] '{}' ({} bits)", off, code, bits);
    }
    println!("  小计: {} 个", tail_zone.len());

    // 6. 对每个合法 code offset，尝试读 item compact header (flags + ver + mode + loc + x + y + page + huffman)
    println!("\n--- 对每个合法 code 尝试 parse compact header ---");
    for (code_offset, _code, _bits) in &valid_codes {
        // code_offset 是 byte-aligned 位置。compact header 在 code 之前 32 bits
        // 所以我们从 code_offset - 4 开始读（如果 code_offset >= 4）
        if *code_offset < 4 { continue; }
        let probe_start = code_offset - 4; // 32-bit flags
        if probe_start + 20 > data.len() { continue; }
        let mut r = BitReader::new(&data[probe_start..]);
        
        let flags = r.read_u32(32);
        let version_bits = r.read_u8(3);
        let mode = r.read_u8(3);
        let location = r.read_u8(4);
        let x = r.read_u8(4);
        let y = r.read_u8(4);
        let page = r.read_u8(3);
        let scanned_code = decode_huffman_string(&mut r).trim().to_string();
        
        let simple = (flags >> 21) & 1;
        let socketed = (flags >> 11) & 1;
        
        println!("  [0x{:04X}] code='{}' flags=0x{:08X} simple={} socketed={} ver={} mode={} loc={} x={} y={} page={}",
            code_offset, scanned_code, flags, simple, socketed, version_bits, mode, location, x, y, page);
    }

    // 7. 尝试在名字后区域作为 JM section 解析
    println!("\n--- 尝试将 0x200..0x1000 作为 JM section 解析 ---");
    for start in [0x200usize, 0x300, 0x400, 0x500, 0x600, 0x700, 0x800] {
        let end = (start + 0x800).min(data.len());
        if start >= data.len() { continue; }
        let chunk = &data[start..end];
        if chunk.len() < 4 { continue; }
        let items = parse_item_stream_sequential(chunk, 0, false);
        if !items.is_empty() {
            println!("  0x{:X}..0x{:X}: {} items", start, end, items.len());
            for (i, it) in items.iter().enumerate().take(20) {
                println!("    [{}] code='{}' x={} y={} mode={:?} loc={:?} ilvl={}",
                    i, it.item.code.trim(), it.item.x, it.item.y, it.item.mode, it.item.location, it.item.item_level);
            }
        }
    }
}

/// 全文件 byte-aligned 扫描，找所有合法的 Huffman 物品码（3-char ASCII）
fn scan_all_huffman_codes(data: &[u8]) -> Vec<(usize, String, usize)> {
    let mut results = Vec::new();
    let mut seen_codes = std::collections::HashSet::new();

    // 在每 byte offset 尝试 decode Huffman
    for byte_off in 0..data.len().saturating_sub(4) {
        let mut r = BitReader::new(&data[byte_off..]);
        let code = decode_huffman_string(&mut r);
        let trimmed = code.trim().to_string();
        if trimmed.len() >= 3 && trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
            let bits_consumed = r.offset();
            let key = (byte_off, trimmed.clone());
            if seen_codes.insert(key) {
                results.push((byte_off, trimmed, bits_consumed));
            }
        }
    }

    // 去重相邻近似偏移（同一个 item 可能在连续几个 byte 被扫到）
    results.sort_by_key(|(off, _, _)| *off);
    let mut deduped: Vec<(usize, String, usize)> = Vec::new();
    for r in &results {
        if let Some(last) = deduped.last()
            && r.0.abs_diff(last.0) < 4 && r.1 == last.1 {
                continue;
            }
        deduped.push(r.clone());
    }
    deduped
}
