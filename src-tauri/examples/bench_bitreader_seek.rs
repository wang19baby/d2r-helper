//! 性能测试: BitReader::new(data) vs BitReader::new(data) + seek()
//! 评估 parser.rs 当前模式(每个 item clone 整个 BitVec)的成本
//!
//! 用法: cargo run --release --example bench_bitreader_seek

use d2r_marketplace_lib::core::BitReader;
use std::time::Instant;

fn main() {
    // 模拟 50KB d2s 文件
    let data: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();

    let runs = 100;

    // Baseline: 现有模式 - 每个 item 创建新 BitReader + skip_bits
    let t1 = Instant::now();
    let mut total: u64 = 0;
    for _ in 0..runs {
        for byte_off in (0..data.len() - 100).step_by(50) {
            for bit_off in 0..8 {
                let mut r = BitReader::new(&data[byte_off..]);
                r.skip_bits(bit_off);
                total = total.wrapping_add(r.read_u32(8) as u64);
            }
        }
    }
    let baseline_ms = t1.elapsed().as_millis();
    let baseline_count = data.len() / 50 * 8 * runs;

    // Optimized: seek 模式 (一个 reader,多次 seek)
    let t2 = Instant::now();
    let mut total2: u64 = 0;
    let mut r = BitReader::new(&data);
    for _ in 0..runs {
        for byte_off in (0..data.len() - 100).step_by(50) {
            for bit_off in 0..8 {
                r.seek(byte_off * 8 + bit_off);
                total2 = total2.wrapping_add(r.read_u32(8) as u64);
            }
        }
    }
    let optimized_ms = t2.elapsed().as_millis();

    println!("=== BitReader benchmark ===");
    println!("  Baseline (new + skip_bits): {} ms ({} iterations, total={})", baseline_ms, baseline_count, total);
    println!("  Optimized (single + seek):   {} ms (total={})", optimized_ms, total2);
    println!("  Speedup: {:.2}x", baseline_ms as f64 / optimized_ms.max(1) as f64);
    println!("\n注意:结果应一致,否则 seek() 有 bug");

    // 验证业务逻辑不变
    assert_eq!(total, total2, "BUG: total != total2 — seek() 行为不同");
    println!("\n✓ 行为一致");
}
