//! TC59 Ragnarok + 4 socketed Facets 详细 trace
//!
//! 真实格式: Ragnarok (Unique Bone Visage) 4 socketed Heaven Facets (jew)
//! 期望: parse_file 应该解出 1 main + 4 socketed = 5 items
//! 当前实测: 2 items (main + 1)

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const TC59: &str = "D:/work_space/personal_workspace/d2r/d2i-research/d2rr-toolkit/tests/cases/TC59/StressTest.d2i";

#[test]
fn test_tc59_detail_trace() {
    if !std::path::Path::new(TC59).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", TC59);
        return;
    }
    let bytes = std::fs::read(TC59).expect("read TC59");
    let file = parse_file(&bytes).expect("parse TC59");

    println!("TC59 page count = {}, items = {}", file.pages.len(), file.items.len());
    for p in file.pages.iter() {
        let pd = p.data.as_slice();
        let jm = if pd.len() >= 4 && &pd[0x40..0x42] == b"JM" {
            u16::from_le_bytes([pd[0x42], pd[0x43]]) as usize
        } else { 0 };
        println!("  Page[{}] jm={} page_size={}", p.index, jm, p.size);
    }

    println!("\nTC59 parsed items:");
    for (i, pi) in file.items.iter().enumerate() {
        println!("  [{}] bit={} code='{}' x={} y={} q={:?} ns={} sockets={}",
            i, pi.raw_bit_offset, pi.item.code.trim(),
            pi.item.x, pi.item.y, pi.item.quality,
            pi.item.num_sockets,
            pi.item.socketed_items.len());
    }

    // Find main Ragnarok (Bone Visage = 'bwn')
    if let Some(main) = file.items.iter().find(|p| p.item.code.trim() == "bwn") {
        println!("\nMain Ragnarok 'bwn':");
        println!("  sockets declared: {}", main.item.num_sockets);
        println!("  sockets parsed: {}", main.item.socketed_items.len());
        for (i, s) in main.item.socketed_items.iter().enumerate() {
            println!("    socket[{}] code='{}' q={:?}",
                i, s.code.trim(), s.quality);
        }
    }
}
