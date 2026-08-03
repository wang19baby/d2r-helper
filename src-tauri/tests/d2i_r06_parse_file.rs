//! 调查为什么 r06 amount 解析成 0

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::protocol::d2i::page::split_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const REAL_GAME_D2I: &str = "D:/work_space/personal_workspace/d2r/ModernSharedStashSoftCoreV2.d2i";

#[test]
fn test_r06_via_parse_file() {
    if !std::path::Path::new(REAL_GAME_D2I).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REAL_GAME_D2I);
        return;
    }
    let bytes = std::fs::read(REAL_GAME_D2I).expect("real game d2i");
    let file = parse_file(&bytes).expect("parse");

    let page5: Vec<_> = file.items.iter()
        .filter(|it| it.page_index == 5)
        .filter(|it| it.item.code.trim() == "r06")
        .collect();
    println!("r06 in Page[5]: {}", page5.len());
    for p in &page5 {
        println!(
            "  bit_off={} bit_len={} amount={}",
            p.raw_bit_offset, p.raw_bit_length, p.item.amount
        );
    }

    // 直接 trace bit 流
    let (pages, _) = split_pages(&bytes).expect("split");
    let page5_data = pages[5].item_bytes();
    let mut r = BitReader::new(page5_data);
    let _ = r.read_string(2);
    let count = r.read_u16(16);
    let body_start = r.offset();
    println!("\nPage[5] body_start={}, count={}", body_start, count);

    // 找 r06
    let mut probe = (body_start + 7) & !7;
    while probe + 32 + 9 <= page5_data.len() * 8 {
        let mut rr = BitReader::new(page5_data);
        rr.seek(probe);
        let fr = rr.read_u32(32);
        let simple = (fr >> 21) & 1;
        if simple == 1 {
            rr.seek(probe + 32);
            let _v = rr.read_u8(3);
            let _m = rr.read_u8(3);
            let _l = rr.read_u8(4);
            let _x = rr.read_u8(4);
            let _y = rr.read_u8(4);
            let _p = rr.read_u8(3);
            let code = decode_huffman_string(&mut rr);
            if code.trim() == "r06" {
                println!("\nr06 at bit {} (simple=1)", probe);
                let _ns = rr.read_u8(1);
                let cs = rr.read_bit();
                println!("  cs flag: {}", cs);
                if cs == 1 {
                    let amt = rr.read_u8(8);
                    println!("  amount: {}", amt);
                }
                return;
            }
        }
        probe += 8;
    }
}
