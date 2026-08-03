#![cfg(any())]

//! Huffman 4-char code 严格扫描 — 在 d2s 整个文件所有 bit 位置找 rin/amu/vip
//! 用 d2i 共享的 BitReader + Huffman decoder。

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;

const RING_AMULET_CODES: &[&str] = &["rin", "amu", "vip"];

/// 严格 byte-aligned 扫描: 只在 byte 边界 (0/8/16/... bit) 尝试解码。
fn scan_byte_aligned(data: &[u8]) -> Vec<(usize, String)> {
    let mut matches = Vec::new();
    for byte_off in 0..data.len().saturating_sub(4) {
        let mut reader = BitReader::new(&data[byte_off..]);
        let code = decode_huffman_string(&mut reader);
        if RING_AMULET_CODES.contains(&code.as_str()) {
            let end_bit = reader.offset();
            if end_bit % 8 == 0 {
                matches.push((byte_off, code));
            }
        }
    }
    matches
}

/// 严格 byte-aligned + 0x20 padding + 12B 装备结构
fn scan_byte_aligned_strict(data: &[u8]) -> Vec<(usize, String, [u8; 8])> {
    let mut matches = Vec::new();
    for byte_off in 0..data.len().saturating_sub(12) {
        let mut reader = BitReader::new(&data[byte_off..]);
        let code = decode_huffman_string(&mut reader);
        if !RING_AMULET_CODES.contains(&code.as_str()) { continue; }
        let end_bit = reader.offset();
        if end_bit % 8 != 0 { continue; }
        let cur = byte_off + end_bit / 8;
        if cur >= data.len() || data[cur] != 0x20 { continue; }
        if cur + 8 > data.len() { continue; }
        let item = &data[cur + 1..cur + 8];
        let i_lvl = item[0];
        let quality = item[1];
        if i_lvl == 0 { continue; }
        if !(1..=8).contains(&quality) { continue; }
        if item[2..].iter().all(|&b| b == 0) { continue; }
        let mut combined = [0u8; 8];
        combined[0] = i_lvl;
        combined[1] = quality;
        combined[2..].copy_from_slice(&item[2..]);
        matches.push((byte_off, code, combined));
    }
    matches
}

#[test]
fn byte_aligned_scan_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let m = scan_byte_aligned(&data);
    println!("happy_manman.d2s: {} byte-aligned rin/amu/vip matches", m.len());
    for (byte_off, code) in &m {
        println!("  byte=0x{:04x} code={}", byte_off, code);
    }
    assert!(m.is_empty(), "expected no ring/amulet in happy_manman.d2s (0x12B..0xF47 mod area)");
}

#[test]
fn byte_aligned_scan_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let m = scan_byte_aligned(&data);
    println!("xieedi.d2s: {} byte-aligned rin/amu/vip matches", m.len());
    for (byte_off, code) in &m {
        println!("  byte=0x{:04x} code={}", byte_off, code);
    }
    assert!(m.is_empty(), "expected no ring/amulet in xieedi.d2s");
}

#[test]
fn strict_scan_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let m = scan_byte_aligned_strict(&data);
    println!("happy_manman.d2s: {} strict 12B-item matches", m.len());
    for (byte_off, code, item) in &m {
        let hex: String = item.iter().map(|b| format!("{:02x}", b)).collect();
        println!("  byte=0x{:04x} code={} item={}", byte_off, code, hex);
    }
    assert!(m.is_empty(), "no full item structure matches");
}

#[test]
fn strict_scan_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let m = scan_byte_aligned_strict(&data);
    println!("xieedi.d2s: {} strict 12B-item matches", m.len());
    for (byte_off, code, item) in &m {
        let hex: String = item.iter().map(|b| format!("{:02x}", b)).collect();
        println!("  byte=0x{:04x} code={} item={}", byte_off, code, hex);
    }
    assert!(m.is_empty(), "no full item structure matches");
}
