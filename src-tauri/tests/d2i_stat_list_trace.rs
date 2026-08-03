//! Page[0] 前 3 个 item 完整 bit 流追踪 — 定位 0x1FF 终止位错位
//!
//! 已知问题: v3 scan + count-based 都只解出 ~45/80,真实 mod 物品应都是 3-char 合法 code
//!
//! 目标: 手工解码 item[0..2] 的完整 stat_list 位流,定位 0x1FF 终止位
//!
//! 用法: cargo test --test d2i_stat_list_trace -- --nocapture

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::protocol::d2i::page::split_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

/// 1. 找 Page[0] 中前 3 个 item 的 bit 起点
#[test]
fn test_find_item_offsets() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i must exist");
    let (pages, _) = split_pages(&bytes).expect("split_pages");
    let page0 = pages.first().expect("page 0");
    let item_data = page0.item_bytes();

    println!("\n=== Page[0] item_data 总长: {} bytes ({} bits) ===", item_data.len(), item_data.len() * 8);

    // 跳过 32-bit JM header (4 bytes)
    let mut r = BitReader::new(item_data);
    let jm = r.read_string(2);
    let count = r.read_u16(16);
    println!("JM magic: {:?}, count: {}", jm, count);
    let body_start = r.offset();
    println!("body_start bit: {}", body_start);

    // 第一个 item 起点 = body_start
    println!("\nitem[0] body 起点: bit {} (byte {}.{})", body_start, body_start / 8, body_start % 8);

    // 用 parser 解
    let file = parse_file(&bytes).expect("parse");
    let page0_items: Vec<_> = file.items.iter().filter(|it| it.page_index == 0).collect();
    println!("\n=== parser 解出 Page[0] 前 5 个 item ===");
    for (i, p) in page0_items.iter().take(5).enumerate() {
        println!(
            "  [{}] code='{}' x={} y={} q={:?} bit_off={} bit_len={} stat_lists={}",
            i, p.item.code.trim(), p.item.x, p.item.y, p.item.quality,
            p.raw_bit_offset, p.raw_bit_length, p.item.stat_lists.len()
        );
        for (j, sl) in p.item.stat_lists.iter().enumerate() {
            println!("      stat_list[{}] ({} stats):", j, sl.stats.len());
            for s in sl.stats.iter().take(5) {
                println!("        stat id={} value={}", s.id, s.value);
            }
        }
    }
}

/// 2. 手工解析 item[0] (hbw) 完整 stat_list
#[test]
fn test_trace_item0_hbw_stat_list() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i must exist");
    let (pages, _) = split_pages(&bytes).expect("split_pages");
    let page0 = pages.first().expect("page 0");
    let item_data = page0.item_bytes();

    // item[0] 起点: body_start = 32 (跳过 4 bytes JM)
    let start: usize = 32;
    let mut r = BitReader::new(item_data);
    r.seek(start);

    println!("\n=== 手工追踪 item[0] (hbw) 完整 stat_list ===\n");

    // 32-bit flags
    let fr = r.read_u32(32);
    let simple = (fr >> 21) & 1;
    let identified = (fr >> 4) & 1;
    let socketed = (fr >> 11) & 1;
    println!("flags=0x{:08X} identified={} socketed={} simple={}", fr, identified, socketed, simple);

    // compact header
    let ver = r.read_u8(3);
    let mode = r.read_u8(3);
    let loc = r.read_u8(4);
    let x = r.read_u8(4);
    let y = r.read_u8(4);
    let page = r.read_u8(3);
    println!("compact: ver={} mode={} loc={} x={} y={} page={}", ver, mode, loc, x, y, page);

    // Huffman code
    let code = decode_huffman_string(&mut r);
    println!("code='{}' (huffman end at bit {})", code.trim(), r.offset());

    // 3-bit socket count (non-simple)
    let ns = r.read_u8(3);
    println!("num_sockets={} (read at bit {})", ns, r.offset());

    // 32-bit item id
    let id = r.read_u32(32);
    println!("item_id=0x{:08X} (read at bit {})", id, r.offset());

    // 7-bit item level
    let ilvl = r.read_u8(7);
    println!("item_level={} (read at bit {})", ilvl, r.offset());

    // 4-bit quality
    let q = r.read_u8(4);
    println!("quality={} (read at bit {})", q, r.offset());

    // 1-bit multi_pic
    let mp = r.read_bit();
    println!("multi_pic={} (read at bit {})", mp, r.offset());
    if mp == 1 {
        let pid = r.read_u8(3);
        println!("  pic_id={}", pid);
    }

    // 1-bit class_specific
    let cs = r.read_bit();
    println!("class_specific={} (read at bit {})", cs, r.offset());
    if cs == 1 {
        let cs_id = r.read_u16(11);
        println!("  class_id={}", cs_id);
    }

    // quality=2 (Normal): 无 quality-specific data
    // 但可能有 realm_data
    let realm = r.read_bit();
    println!("realm_data={} (read at bit {})", realm, r.offset());
    if realm == 1 {
        r.skip_bits(96);
        println!("  skipped 96 bits realm");
    }

    // hbw 是 weapon → 读 max_dur + cur_dur + 2 unk
    let md = r.read_u8(8);
    println!("max_dur={} (read at bit {})", md, r.offset());
    if md > 0 {
        let cd = r.read_u8(8);
        println!("cur_dur={}", cd);
        r.skip_bits(2);
        println!("  + 2 bits unknown");
    }

    // ★ stat_list 起点
    let stat_list_start = r.offset();
    println!("\n★ stat_list 起点: bit {}", stat_list_start);

    // 逐个 stat 读取,看 0x1FF 终止位
    let mut i = 0;
    let mut found_0x1ff = false;
    while r.offset() + 9 <= item_data.len() * 8 {
        let stat_id = r.read_u16(9);
        if stat_id == 0x1FF {
            println!("  [{}] stat_id=0x1FF (TERMINATOR) at bit {}", i, r.offset() - 9);
            found_0x1ff = true;
            break;
        }
        // 假设是简单 stat (无 param,无 sub-prop) 8 bits value
        let val_bits = match stat_id {
            0..=6 => 10,  // str/dex/vit/energy/statpts/skillpts
            7..=11 => 21,  // hp/mana/stam
            12 => 7,  // level
            13 => 32,  // exp
            14 | 15 => 25,  // gold
            _ => 8,  // 默认
        };
        let val = r.read_u32(val_bits);
        if i < 10 {
            println!("  [{}] stat_id={} value={} (read at bit {})", i, stat_id, val, r.offset());
        }
        i += 1;
    }
    if !found_0x1ff {
        println!("  ! NOT FOUND 0x1FF in remaining bits");
    }
    let stat_list_end = r.offset();
    println!("\n★ stat_list 终点: bit {} (长度 {} bits = {} bytes)", stat_list_end, stat_list_end - stat_list_start, (stat_list_end - stat_list_start) / 8);

    // parser 解出的 item[0] raw_bit_length
    let file = parse_file(&bytes).expect("parse");
    let p0 = file.items.iter().find(|p| p.page_index == 0).expect("item 0");
    println!("\nparser 报告 item[0]: bit_off={} bit_len={}", p0.raw_bit_offset, p0.raw_bit_length);
    println!("parser 算出 item 终点 bit: {}", p0.raw_bit_offset + p0.raw_bit_length);
}

/// 3. 找到所有 main magic properties 后的 0x1FF,看是否漏读
#[test]
fn test_find_0x1ff_in_page0() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i must exist");
    let (pages, _) = split_pages(&bytes).expect("split_pages");
    let page0 = pages.first().expect("page 0");
    let item_data = page0.item_bytes();

    println!("\n=== Page[0] 全量 0x1FF 终止位搜索 ===");
    println!("item_data total: {} bytes = {} bits", item_data.len(), item_data.len() * 8);

    // 跳过 32-bit JM header
    let mut r = BitReader::new(item_data);
    r.read_string(2);
    r.read_u16(16);
    let body_start = r.offset();

    let mut r2 = BitReader::new(item_data);
    let mut found_offsets: Vec<usize> = Vec::new();
    let mut probe = body_start;
    while probe + 9 <= item_data.len() * 8 {
        r2.seek(probe);
        let v = r2.read_u16(9);
        if v == 0x1FF {
            found_offsets.push(probe);
        }
        probe += 1;
    }
    println!("Page[0] 中 0x1FF 出现 {} 次 (byte-aligned probe):", found_offsets.len());
    for o in &found_offsets {
        println!("  bit {} (byte {})", o, o / 8);
    }

    // parser 解出每个 item 终点
    let file = parse_file(&bytes).expect("parse");
    let page0_items: Vec<_> = file.items.iter().filter(|it| it.page_index == 0).collect();
    println!("\nparser 解出 item 终点:");
    for (i, p) in page0_items.iter().enumerate().take(10) {
        let end = p.raw_bit_offset + p.raw_bit_length;
        let near_0x1ff = found_offsets.iter().any(|o| (*o as i64 - end as i64).abs() < 16);
        println!(
            "  [{}] code='{}' end_bit={} (byte {}) {}",
            i, p.item.code.trim(), end, end / 8,
            if near_0x1ff { "← 0x1FF 附近" } else { "← 远离 0x1FF (错位?)" }
        );
    }
}
