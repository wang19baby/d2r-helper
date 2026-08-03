//! 真实 d2i/d2s 解析性能基准 (US-026 验证)
//!
//! 用 xieedi.d2s 真实 d2s 文件 + ModernSharedStashSoftCoreV2.d2i 真实 d2i 文件作为 benchmark。
//! US-026 重构后预期 1.5x+ 加速(单点 BitReader 测得 2258x,但实际有 stat/quality 解析开销)。
//!
//! 用法: cargo test --release --test d2i_parse_bench -- --nocapture

use d2r_marketplace_lib::protocol::d2i::parser::parse_file as parse_d2i;
use d2r_marketplace_lib::protocol::d2s::parse_file as parse_d2s;
use std::path::PathBuf;
use std::time::Instant;

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    manifest_dir.join("tests").join("fixtures")
}

fn largest_d2s(dir: &PathBuf) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "d2s").unwrap_or(false))
        .max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
}

fn largest_d2i(dir: &PathBuf) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "d2i").unwrap_or(false))
        .max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
}

#[test]
fn bench_parse_d2s_per_file() {
    let dir = fixtures_dir();
    let target = match largest_d2s(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2s fixture"); return; }
    };

    let bytes = std::fs::read(&target).expect("read d2s");
    eprintln!("\n[d2i_parse_bench:d2s] file: {} ({} bytes)",
        target.file_name().unwrap().to_string_lossy(), bytes.len());

    // Warmup
    let _ = parse_d2s(&bytes);

    let runs = 10;
    let mut total = std::time::Duration::ZERO;
    for _ in 0..runs {
        let t = Instant::now();
        let result = parse_d2s(&bytes);
        let elapsed = t.elapsed();
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        total += elapsed;
    }
    let avg = total / runs;
    eprintln!("[d2i_parse_bench:d2s] parse_d2s avg: {:.2} ms ({} runs)", avg.as_millis(), runs);

    // 防止编译器优化掉
    eprintln!("[d2i_parse_bench:d2s] result OK, items parsed");
}

/// ★ US-026 核心验证: parse_d2i 在真实 d2i fixture 上的整体耗时
/// 重构前 BitVec::from_slice 在每个 item 边界各触发一次 (800K iterations 54s 测得)
/// 重构后预期 1.5x+ 加速(单 item seek 路径 2258x,但实际 stat/quality 解析有开销)
#[test]
fn bench_parse_d2i_per_file() {
    let dir = fixtures_dir();
    let target = match largest_d2i(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2i fixture"); return; }
    };

    let bytes = std::fs::read(&target).expect("read d2i");
    eprintln!("\n[d2i_parse_bench:d2i] file: {} ({} bytes)",
        target.file_name().unwrap().to_string_lossy(), bytes.len());

    // Warmup
    let _ = parse_d2i(&bytes);

    let runs = 20;
    let mut total = std::time::Duration::ZERO;
    for _ in 0..runs {
        let t = Instant::now();
        let result = parse_d2i(&bytes);
        let elapsed = t.elapsed();
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        total += elapsed;
    }
    let avg = total / runs;
    eprintln!("[d2i_parse_bench:d2i] parse_d2i avg: {:.2} ms ({} runs)", avg.as_millis(), runs);

    eprintln!("[d2i_parse_bench:d2i] result OK, pages parsed");
}

/// 业务逻辑不变:parse_d2s 解析结果在重构前后必须一致
/// 这里只检查不 panic + 物品总数 > 0
#[test]
fn test_parse_d2s_correctness_smoke() {
    let dir = fixtures_dir();
    let target = match largest_d2s(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2s fixture"); return; }
    };
    let bytes = std::fs::read(&target).expect("read d2s");

    let result = parse_d2s(&bytes).expect("parse ok");
    let total = result.equipped.len() + result.backpack.len() + result.belt.len()
        + result.cube.len() + result.merc.len() + result.personal_stash.len();
    eprintln!("[parse_d2s_smoke] total items: {} (eq={} bp={} belt={} cube={} merc={} stash={})",
        total, result.equipped.len(), result.backpack.len(), result.belt.len(),
        result.cube.len(), result.merc.len(), result.personal_stash.len());
    assert!(total > 0, "parse should produce items");
}

/// 业务逻辑不变:parse_d2i 解析结果在重构前后必须一致
#[test]
fn test_parse_d2i_correctness_smoke() {
    let dir = fixtures_dir();
    let target = match largest_d2i(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2i fixture"); return; }
    };
    let bytes = std::fs::read(&target).expect("read d2i");

    let result = parse_d2i(&bytes).expect("parse ok");
    let total = result.items.len();
    eprintln!("[parse_d2i_smoke] total items: {} (pages={})",
        total, result.pages.len());
    assert!(total > 0, "parse_d2i should produce items");
}
