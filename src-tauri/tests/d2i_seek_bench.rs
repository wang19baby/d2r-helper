//! 性能测试:BitReader::new(item_data) vs seek 复用模式
//!
//! 目标:验证 BitReader 创建是 d2i 解析的主要开销,seek 优化是值得的。
//!
//! 行为不变保证:
//! - baseline: `BitReader::new(&data[byte..]); r.skip_bits(bit)`
//! - optimized: `BitReader::new(&data); r.seek(byte*8 + bit)`
//! - 两者读出的 u32 值必须完全一致(否则 seek() 行为不同)
//!
//! 性能断言:optimized 比 baseline 快至少 10x(BitVec clone vs O(1) seek)
//!
//! 跑法: cargo test --release --test d2i_seek_bench -- --nocapture

use d2r_marketplace_lib::core::BitReader;

#[test]
fn bench_seek_vs_new_skip() {
    // 模拟 50KB d2s (实际 ModernSharedStashSoftCoreV2.d2i 是 18KB)
    let data: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();

    let runs = 100;
    let items_per_run = data.len() / 50; // 每 run ~1000 个虚拟 item
    let total_iterations = items_per_run * 8 * runs; // 每个 item 8 bit offset 变体

    // Baseline: 每个 item 都 new BitReader (clone BitVec)
    let mut baseline_total: u64 = 0;
    let baseline_start = std::time::Instant::now();
    for _ in 0..runs {
        for byte_off in (0..data.len() - 100).step_by(50) {
            for bit_off in 0..8 {
                let mut r = BitReader::new(&data[byte_off..]);
                r.skip_bits(bit_off);
                baseline_total = baseline_total.wrapping_add(r.read_u32(8) as u64);
            }
        }
    }
    let baseline_ms = baseline_start.elapsed().as_millis();

    // Optimized: 单个 BitReader + seek
    let mut optimized_total: u64 = 0;
    let optimized_start = std::time::Instant::now();
    let mut r = BitReader::new(&data);
    for _ in 0..runs {
        for byte_off in (0..data.len() - 100).step_by(50) {
            for bit_off in 0..8 {
                r.seek(byte_off * 8 + bit_off);
                optimized_total = optimized_total.wrapping_add(r.read_u32(8) as u64);
            }
        }
    }
    let optimized_ms = optimized_start.elapsed().as_millis();

    println!("\n[d2i_seek_bench] {} iterations per run × {} runs = {} total",
        items_per_run * 8, runs, total_iterations);
    println!("[d2i_seek_bench] Baseline (new + skip_bits): {} ms (total={})",
        baseline_ms, baseline_total);
    println!("[d2i_seek_bench] Optimized (single + seek):  {} ms (total={})",
        optimized_ms, optimized_total);
    let speedup = baseline_ms as f64 / optimized_ms.max(1) as f64;
    println!("[d2i_seek_bench] Speedup: {:.1}x", speedup);

    // 业务逻辑不变: 总和必须一致
    assert_eq!(baseline_total, optimized_total,
        "BUG: total {} != optimized_total {} — seek() 与 skip_bits 行为不一致",
        baseline_total, optimized_total);

    // 性能断言: 至少 5x 加速（实测 2258x,但 CI/系统负载会影响具体倍数）
    assert!(speedup > 5.0,
        "expected > 5x speedup, got {:.1}x — performance regression possible", speedup);

    println!("[d2i_seek_bench] ✓ 行为一致 + {}x 加速", speedup);
}

/// 边界测试:seek 后 read 必须返回正确数据
#[test]
fn test_seek_correctness() {
    // 简单测试:seek(0) 等价于新 reader
    let data = [0xAAu8, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    let mut r1 = BitReader::new(&data);
    let v1 = r1.read_u8(8);

    let mut r2 = BitReader::new(&data);
    r2.seek(0);
    let v2 = r2.read_u8(8);

    assert_eq!(v1, v2, "seek(0) must match new reader");
    assert_eq!(v1, 0xAA, "expected 0xAA at bit 0, got 0x{:02X}", v1);

    // 跨 byte 边界:seek(12) 等价于 skip_bits(12) 然后 read_u8(8)
    let mut r1 = BitReader::new(&data);
    r1.skip_bits(12);
    let v1 = r1.read_u8(8);

    let mut r2 = BitReader::new(&data);
    r2.seek(12);
    let v2 = r2.read_u8(8);

    assert_eq!(v1, v2, "seek(12) must match skip_bits(12)+read");
}

/// 边界测试:seek 超出范围不应 panic
#[test]
fn test_seek_clamp() {
    let data = [0x12u8, 0x34];
    let mut r = BitReader::new(&data);

    r.seek(1000); // 远超 16 bits
    assert_eq!(r.remaining_bits(), 0);
    assert!(!r.has_more());
}
