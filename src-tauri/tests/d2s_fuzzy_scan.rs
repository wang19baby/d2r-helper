#![cfg(any())]

//! 完整穷尽性扫描: 用 mod 自定义 + 任意长度 Huffman code 找戒指项链。
//! 不限制 code 长度 (3/4/5 字符都行), 不限制 byte-aligned。

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;

const RING_AMULET_CODES: &[&str] = &["rin", "amu", "vip"];

/// 在 d2s 整个文件每个 bit 偏移解码 Huffman 4-char, 找 3-字符子串匹配 rin/amu/vip
fn scan_fuzzy(data: &[u8]) -> Vec<(usize, String, String)> {
    let mut matches = Vec::new();
    for byte_off in 0..data.len().saturating_sub(4) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            // 解码 4-char Huffman
            let decoded = decode_huffman_string(&mut reader);
            // 必须 byte-aligned: bit_off == 0
            if bit_off != 0 { continue; }
            if decoded.len() == 4 {
                // 4 字符全匹配 (mod 自定义 4-char code)
                if RING_AMULET_CODES.contains(&decoded.as_str()) {
                    matches.push((byte_off * 8 + bit_off, decoded.clone(), decoded.clone()));
                }
                // 3 字符子串 (位置 0 或 1)
                for i in 0..=1 {
                    let sub3 = decoded[i..i+3].to_string();
                    if RING_AMULET_CODES.contains(&sub3.as_str()) {
                        matches.push((byte_off * 8 + bit_off, sub3, decoded.clone()));
                    }
                }
            }
        }
    }
    matches
}

#[test]
fn fuzzy_scan_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let mut m = scan_fuzzy(&data);
    m.sort_by_key(|x| x.0);
    // 去重
    m.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    println!("happy_manman.d2s: {} fuzzy matches", m.len());
    for (bit_off, sub, full) in &m {
        println!("  bit=0x{:04x} sub={} full_decode='{}'", bit_off, sub, full);
    }
}

#[test]
fn fuzzy_scan_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let mut m = scan_fuzzy(&data);
    m.sort_by_key(|x| x.0);
    m.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    println!("xieedi.d2s: {} fuzzy matches", m.len());
    for (bit_off, sub, full) in &m {
        println!("  bit=0x{:04x} sub={} full_decode='{}'", bit_off, sub, full);
    }
}
