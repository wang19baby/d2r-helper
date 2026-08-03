//! 位级追踪分析: 给定字节序列解码 item header
use d2r_marketplace_lib::core::{BitReader};
use d2r_marketplace_lib::core::encoding::decode_huffman_string;

const DATA: [u8; 32] = [
    0x10, 0x00, 0x80, 0x00, 0x05, 0x68, 0x74, 0x19,
    0x1C, 0xAE, 0x43, 0xFE, 0xE6, 0xAB, 0x87, 0x16,
    0x68, 0xC4, 0x8D, 0x0A, 0x13, 0x5A, 0xF8, 0x20,
    0x0D, 0x38, 0x33, 0x44, 0x99, 0xFF, 0x00, 0x00,
];

#[test]
fn bit_trace_analysis() {
    let mut r = BitReader::new(&DATA);
    println!("=== 位级追踪分析 ===");
    println!("数据: 32 bytes");
    println!();

    // Step 0-31: 32b flags
    let flags_raw = r.read_u32(32);
    let ident = (flags_raw >> 4) & 1;
    let sock = (flags_raw >> 11) & 1;
    let ear = (flags_raw >> 16) & 1;
    let simple = (flags_raw >> 21) & 1;
    let new_bit = (flags_raw >> 12) & 1;
    let start_bit = (flags_raw >> 17) & 1;
    let eth = (flags_raw >> 22) & 1;
    let pers = (flags_raw >> 24) & 1;
    let rw = (flags_raw >> 26) & 1;

    println!("[0-31]  flags: 0x{:08x} ident={} sock={} ear={} simple={} new={} start={} eth={} pers={} rw={}",
        flags_raw, ident, sock, ear, simple, new_bit, start_bit, eth, pers, rw);
    println!("         → identified={}, socketed={}, is_ear={}, simple_item={}",
        ident, sock, ear, simple);
    println!();

    // Step 32-34: 3b version
    let bo = r.offset();
    let ver = r.read_u8(3);
    println!("[32-34]  version={} ({}-bits) offset={}b/{}b", ver, 3, bo, r.offset());

    // Step 35-37: 3b mode
    let bo = r.offset();
    let mode = r.read_u8(3);
    println!("[35-37]  mode={} ({}-bits) offset={}b/{}b", mode, 3, bo, r.offset());

    // Step 38-41: 4b equipped
    let bo = r.offset();
    let equipped = r.read_u8(4);
    println!("[38-41]  equipped={} ({}-bits) offset={}b/{}b", equipped, 4, bo, r.offset());

    // Step 42-45: 4b x
    let bo = r.offset();
    let x = r.read_u8(4);
    println!("[42-45]  x={} ({}-bits) offset={}b/{}b", x, 4, bo, r.offset());

    // Step 46-49: 4b y
    let bo = r.offset();
    let y = r.read_u8(4);
    println!("[46-49]  y={} ({}-bits) offset={}b/{}b", y, 4, bo, r.offset());

    // Step 50-52: 3b page
    let bo = r.offset();
    let page = r.read_u8(3);
    println!("[50-52]  page={} ({}-bits) offset={}b/{}b", page, 3, bo, r.offset());

    // Step 53+: Huffman code (variable bits, 4 chars)
    let bo = r.offset();
    let code = decode_huffman_string(&mut r);
    println!("[53..]   huffman code='{}' ({}-bits) offset={}b/{}b", code.trim(), r.offset()-bo, bo, r.offset());

    // Num sockets (非simple = 3 bits)
    let bo = r.offset();
    let ns_bits = if simple == 1 { 1 } else { 3 };
    let ns = r.read_u8(ns_bits);
    println!("[{}..] num_sockets={} ({}-bits) offset={}b/{}b", bo, ns, ns_bits, bo, r.offset());

    // If non-simple, read item_id(32b) + level(7b) + quality(4b)
    if simple == 0 {
        println!("\n=== Non-simple body ===");
        let bo = r.offset();
        let item_id = r.read_u32(32);
        println!("[{}..] item_id=0x{:08x} (32b) offset={}b", bo, item_id, r.offset());
        let bo = r.offset();
        let level = r.read_u8(7);
        println!("[{}..] level={} (7b) offset={}b", bo, level, r.offset());
        let bo = r.offset();
        let quality = r.read_u8(4);
        println!("[{}..] quality={} (4b) offset={}b", bo, quality, r.offset());

        // Multi-pic
        let bo = r.offset();
        let mp = r.read_bit();
        println!("[{}..] multi_pic={} (1b) offset={}b", bo, mp, r.offset());
        if mp == 1 {
            let pid = r.read_u8(3);
            println!("[{}..] pid={} (3b) offset={}b", r.offset()-3, pid, r.offset());
        }

        // Class specific
        let bo = r.offset();
        let cs_bit = r.read_bit();
        println!("[{}..] class_specific_bit={} (1b) offset={}b", bo, cs_bit, r.offset());
        if cs_bit == 1 {
            let cs_id = r.read_u16(11);
            println!("[{}..] class_specific_id={} (11b) offset={}b", r.offset()-11, cs_id, r.offset());
        }

        // Quality-specific fields
        println!("\n--- quality-specific (quality={}) ---", quality);
        match quality {
            1/*Low*/ => {
                let bo = r.offset();
                let v = r.read_u8(3);
                println!("[{}..] low_quality_ext={} (3b) offset={}b", bo, v, r.offset());
            }
            2/*Normal*/ => println!("(Normal: no extra fields)"),
            3/*Superior*/ => {
                let bo = r.offset();
                let v = r.read_u8(3);
                println!("[{}..] superior_ext={} (3b) offset={}b", bo, v, r.offset());
            }
            4/*Magic*/ => {
                let bo = r.offset();
                let p1 = r.read_u16(11);
                let p2 = r.read_u16(11);
                println!("[{}..] magic_prefix={} suffix={} (11+11b) offset={}b", bo, p1, p2, r.offset());
            }
            5/*Set*/ => {
                let bo = r.offset();
                let s = r.read_u16(12);
                println!("[{}..] set_id={} (12b) offset={}b", bo, s, r.offset());
            }
            6/*Rare*/ | 8/*Crafted*/ => {
                let bo = r.offset();
                let n1 = r.read_u8(8);
                let n2 = r.read_u8(8);
                println!("[{}..] rare_name={} {} (8+8b) offset={}b", bo, n1, n2, r.offset());
                for i in 0..6 {
                    let bo = r.offset();
                    let has = r.read_bit();
                    if has == 1 {
                        let v = r.read_u16(11);
                        println!("[{}..] rare_bonus[{}]=has({}) id={} (1+11b) offset={}b", bo, i, has, v, r.offset());
                    } else {
                        println!("[{}..] rare_bonus[{}]=has({}) (1b) offset={}b", bo, i, has, r.offset());
                    }
                }
            }
            7/*Unique*/ => {
                let bo = r.offset();
                let s = r.read_u16(12);
                println!("[{}..] unique_id={} (12b) offset={}b", bo, s, r.offset());
            }
            _ => {
                let bo = r.offset();
                let n1 = r.read_u8(8);
                let n2 = r.read_u8(8);
                println!("[{}..] unknown_quality({}): name={} {} (8+8b) offset={}b", bo, quality, n1, n2, r.offset());
                for _i in 0..6 {
                    let has = r.read_bit();
                    if has == 1 { let _ = r.read_u16(11); }
                }
            }
        }

        // Given runeword
        let bo = r.offset();
        if rw == 1 {
            let _ = r.read_u16(12);
            let _ = r.read_u8(4);
            println!("[{}..] runeword_fields (12+4b) offset={}b", bo, r.offset());
        } else {
            println!("[{}..] no_runeword offset={}b", bo, r.offset());
        }

        // Continue to read remaining data and show position
        let remaining = r.remaining_bits();
        println!("\n--- 剩余数据 ({}/{}b) ---", remaining, DATA.len()*8);
        println!("当前 offset={}b/{}b", r.offset(), DATA.len()*8);
    }

    // Summary
    println!("\n=== 汇总 ===");
    println!("code='{}' (offset 53..{})", code.trim(), r.offset());
    if simple == 0 {
        println!("level @ 85b offset (bit 53+32)", );
    }
}
