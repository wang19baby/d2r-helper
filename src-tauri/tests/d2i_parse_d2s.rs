#![cfg(any())]

//! 用 d2i 的真实 parse_file API 扫 d2s 整个文件, 找所有合法 item 包括戒指项链。
//! d2i parser 内部用 bit-level JM 编码 + Huffman 4-char, 应该能 parse 出标准 d2s/d2i items。

use d2r_marketplace_lib::protocol::d2i::parser::parse_file_grid_order;

#[test]
fn d2i_parse_happy_manman() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("happy_manman.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let res = parse_file_grid_order(&data);
    match res {
        Ok(f) => {
            println!("happy_manman.d2s: {} items total", f.items.len());
            for (i, p) in f.items.iter().enumerate() {
                println!("  [{}] code={} quality={:?} iLvl={} amt={}",
                    i, p.item.code, p.item.quality, p.item.item_level, p.item.amount);
            }
        }
        Err(e) => println!("happy_manman.d2s parse_file_grid_order FAILED: {:?}", e),
    }
}

#[test]
fn d2i_parse_xieedi() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join("xieedi.d2s");
    if !path.exists() { eprintln!("SKIP: fixture 缺失"); return; }
    let data = std::fs::read(&path).expect("read d2s");
    let res = parse_file_grid_order(&data);
    match res {
        Ok(f) => {
            println!("xieedi.d2s: {} items total", f.items.len());
            for (i, p) in f.items.iter().enumerate() {
                println!("  [{}] code={} quality={:?} iLvl={} amt={}",
                    i, p.item.code, p.item.quality, p.item.item_level, p.item.amount);
            }
        }
        Err(e) => println!("xieedi.d2s parse_file_grid_order FAILED: {:?}", e),
    }
}
