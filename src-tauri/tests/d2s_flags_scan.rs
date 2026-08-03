#![cfg(any())]

//! 用 d2i::ItemFlags::read 扫 d2s 整个文件, 找 32b flags + Huffman 4-char code = rin/amu/vip 的位置。
//! ItemFlags 32 bit, 包含 simple_item flag, 之后跟 3b version, mode, location, x, y, page, Huffman code。

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::protocol::common::ItemFlags;

#[test]
fn flags_scan_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");

    let mut rin_amu_vip = 0;
    // 每个 byte 起始 + 每个 bit 偏移
    for byte_off in 0..data.len().saturating_sub(16) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            // 32b ItemFlags
            let Ok(flags) = ItemFlags::read(&mut reader) else { continue; };
            // 简单过滤: 必须 simple_item (戒指项链) 或 socketed (有空间)
            if !flags.simple_item() && !flags.socketed() { continue; }
            // 3b version (v105 = 105 = 0b1101001)
            let version = reader.read_u8(3);
            if version > 7 { continue; }  // v105 modded = 7
            // 3b mode + 4b location + 4b x + 4b y + 3b page = 18b
            // (BitReader 不直接有 read_u8(3) 但 read_u8(3) 等价)
            let mode = reader.read_u8(3);
            let location = reader.read_u8(4);
            let _x = reader.read_u8(4);
            let _y = reader.read_u8(4);
            let _page = reader.read_u8(3);
            // 4-char Huffman code
            let code = decode_huffman_string(&mut reader);
            if code == "rin" || code == "amu" || code == "vip" {
                let end = reader.offset();
                let ring_byte = byte_off + end / 8;
                let ring_sub = end % 8;
                println!(
                    "0x{:04x} (byte 0x{:03x} +{}b) flags=0x{:08x} v={} mode={} loc={} code={}",
                    byte_off * 8 + bit_off, byte_off, bit_off, flags.raw, version, mode, location, code
                );
                rin_amu_vip += 1;
            }
        }
    }
    println!("happy_manman.d2s: {} rin/amu/vip via flags scan", rin_amu_vip);
}

#[test]
fn flags_scan_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");

    let mut rin_amu_vip = 0;
    for byte_off in 0..data.len().saturating_sub(16) {
        for bit_off in 0..8 {
            let mut reader = BitReader::new(&data[byte_off..]);
            for _ in 0..bit_off {
                let _ = reader.read_bit();
            }
            let Ok(flags) = ItemFlags::read(&mut reader) else { continue; };
            if !flags.simple_item() && !flags.socketed() { continue; }
            let version = reader.read_u8(3);
            if version > 7 { continue; }
            let mode = reader.read_u8(3);
            let location = reader.read_u8(4);
            let _x = reader.read_u8(4);
            let _y = reader.read_u8(4);
            let _page = reader.read_u8(3);
            let code = decode_huffman_string(&mut reader);
            if code == "rin" || code == "amu" || code == "vip" {
                let end = reader.offset();
                let ring_byte = byte_off + end / 8;
                let ring_sub = end % 8;
                println!(
                    "0x{:04x} (byte 0x{:03x} +{}b) flags=0x{:08x} v={} mode={} loc={} code={}",
                    byte_off * 8 + bit_off, byte_off, bit_off, flags.raw, version, mode, location, code
                );
                rin_amu_vip += 1;
            }
        }
    }
    println!("xieedi.d2s: {} rin/amu/vip via flags scan", rin_amu_vip);
}
