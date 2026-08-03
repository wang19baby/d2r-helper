//! TC59 socketed item 详细 trace
use d2r_marketplace_lib::core::BitReader;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const TC59: &str = "D:/work_space/personal_workspace/d2r/d2i-research/d2rr-toolkit/tests/cases/TC59/StressTest.d2i";

#[test]
fn test_tc59_socket_trace() {
    if !std::path::Path::new(TC59).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", TC59);
        return;
    }
    let bytes = std::fs::read(TC59).expect("read");
    let file = parse_file(&bytes).expect("parse");
    let main_idx = 0; // uhm (Ragnarok)
    let main = &file.items[main_idx];

    println!("Main Ragnarok 'uhm' ns=4 sockets={}", main.item.socketed_items.len());
    println!("Main item bytes from bit {} to {}", main.raw_bit_offset, main.raw_bit_offset + main.raw_bit_length);

    // 4 socketed items 起点
    for (i, s) in main.item.socketed_items.iter().enumerate() {
        println!("  socket[{}] code='{}' socket_offset={} (would be relative to main start)",
            i, s.code.trim(),
            s.id);
    }

    // 显示 socket 区间 raw bytes
    let main_end_byte = (main.raw_bit_offset + main.raw_bit_length) / 8;
    println!("\nMain end at byte {} (offset in item_data)", main_end_byte);

    // 直接解析 socket[0] 起点 bit=904
    let item_data = file.pages[0].item_bytes();
    let mut r = BitReader::new(item_data);
    r.seek(904);
    let socket0_flags = r.read_u32(32);
    println!("\nSocket[0] @ bit=904, flags=0x{:08X}", socket0_flags);

    // 4 socketed items 的 raw 字节
    println!("\nRaw bytes around socketed items (byte {} - {}):", (main.raw_bit_offset + main.raw_bit_length) / 8 - 5, main_end_byte + 8);
    let start_b = main_end_byte.saturating_sub(2);
    let end_b = (main_end_byte + 30).min(item_data.len());
    for (i, b) in item_data[start_b..end_b].iter().enumerate() {
        if i % 16 == 0 && i > 0 { println!(); }
        print!("{:02x} ", b);
    }
    println!();
}
