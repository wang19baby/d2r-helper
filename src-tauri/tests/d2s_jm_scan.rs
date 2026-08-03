#![cfg(any())]

//! 用 d2i 标准 JM 协议 (32b ItemFlags + Huffman 4-char) 扫 d2s 整个文件找 rin/amu/vip。
//! 不限 byte-aligned, 每个 bit 偏移都试。

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::protocol::common::ItemFlags;
use d2r_marketplace_lib::protocol::d2i::parser::ParsedItem;

const TARGET_CODES: &[&str] = &["rin", "amu", "vip"];

#[test]
fn standard_jm_scan_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");

    let mut matches = 0;
    for byte_off in 0..data.len().saturating_sub(8) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            // 32b ItemFlags
            let Ok(flags) = ItemFlags::read(&mut reader) else { continue; };
            // 简单装备或 socketed
            if !flags.simple_item() && !flags.socketed() { continue; }
            // 3b version
            let version = reader.read_u8(3);
            if version > 7 { continue; }
            // 3b mode + 4b location + 4b x + 4b y + 3b page = 18b
            let _mode = reader.read_u8(3);
            let _location = reader.read_u8(4);
            let _x = reader.read_u8(4);
            let _y = reader.read_u8(4);
            let _page = reader.read_u8(3);
            // 4-char Huffman code
            let code = decode_huffman_string(&mut reader);
            if TARGET_CODES.contains(&code.as_str()) {
                println!(
                    "0x{:04x} (byte 0x{:03x} +{}b) flags=0x{:08x} v={} code={}",
                    byte_off * 8 + bit_off, byte_off, bit_off, flags.raw, version, code
                );
                matches += 1;
            }
        }
    }
    println!("happy_manman.d2s: {} standard JM rin/amu/vip matches", matches);
}

#[test]
fn standard_jm_scan_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");

    let mut matches = 0;
    for byte_off in 0..data.len().saturating_sub(8) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            let Ok(flags) = ItemFlags::read(&mut reader) else { continue; };
            if !flags.simple_item() && !flags.socketed() { continue; }
            let version = reader.read_u8(3);
            if version > 7 { continue; }
            let _mode = reader.read_u8(3);
            let _location = reader.read_u8(4);
            let _x = reader.read_u8(4);
            let _y = reader.read_u8(4);
            let _page = reader.read_u8(3);
            let code = decode_huffman_string(&mut reader);
            if TARGET_CODES.contains(&code.as_str()) {
                println!(
                    "0x{:04x} (byte 0x{:03x} +{}b) flags=0x{:08x} v={} code={}",
                    byte_off * 8 + bit_off, byte_off, bit_off, flags.raw, version, code
                );
                matches += 1;
            }
        }
    }
    println!("xieedi.d2s: {} standard JM rin/amu/vip matches", matches);
}

/// 全面 JM 扫描: 包含所有 3-char code (不只是 rin/amu/vip), 列前 30 个出现最多的
#[test]
fn full_jm_code_frequency() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");

    use std::collections::HashMap;
    let mut freq: HashMap<String, usize> = HashMap::new();
    // 0x12B..0xF47 mod 区域
    for byte_off in 0x12B..0xF47.min(data.len().saturating_sub(8)) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            let Ok(flags) = ItemFlags::read(&mut reader) else { continue; };
            if !flags.simple_item() && !flags.socketed() { continue; }
            let version = reader.read_u8(3);
            if version > 7 { continue; }
            let _mode = reader.read_u8(3);
            let _location = reader.read_u8(4);
            let _x = reader.read_u8(4);
            let _y = reader.read_u8(4);
            let _page = reader.read_u8(3);
            let code = decode_huffman_string(&mut reader);
            if code.len() == 4 {
                // 过滤掉"重复字符 + 噪声"模式
                let unique_chars: std::collections::HashSet<char> = code.chars().collect();
                if unique_chars.len() >= 3 {
                    *freq.entry(code).or_insert(0) += 1;
                }
            }
        }
    }
    let mut v: Vec<_> = freq.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    println!("xieedi 0x12B..0xF47 unique-3+ char codes (potential real items):");
    for (code, n) in v.iter().take(30) {
        let hit_ring = ["rin","amu","vip"].contains(&code.as_str());
        println!("  {} count={} {}", code, n, if hit_ring { "<== RING/AMULET" } else { "" });
    }
}
