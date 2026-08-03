//! Character Loading Command Benchmark
//!
//! 测量 list_characters_brief(dir) 与 parse_d2s(bytes) 的耗时,
//! 用于发现 d2s parser 的性能退化。

use std::path::PathBuf;
use std::time::Instant;

use d2r_marketplace_lib::commands::character::list_characters_brief;
use d2r_marketplace_lib::protocol::d2s::parse_file as parse_d2s;

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    manifest_dir.join("tests").join("fixtures")
}

fn list_d2s_names(dir: &PathBuf) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .expect("read fixtures dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "d2s").unwrap_or(false))
        .collect()
}

#[test]
fn bench_list_characters_brief() {
    let dir = fixtures_dir();
    let d2s_files = list_d2s_names(&dir);
    eprintln!("\n[bench] fixtures dir: {}", dir.display());
    eprintln!("[bench] found {} .d2s files", d2s_files.len());

    let _ = list_characters_brief(dir.to_string_lossy().to_string());

    let mut total_us = 0u128;
    let runs = 5;
    for i in 0..runs {
        let start = Instant::now();
        let result = list_characters_brief(dir.to_string_lossy().to_string());
        let elapsed_us = start.elapsed().as_micros();
        total_us += elapsed_us;
        assert!(result.is_ok(), "list_characters_brief failed");
        eprintln!("[bench] run {}: {} µs", i + 1, elapsed_us);
    }
    let avg_us = total_us / runs as u128;
    eprintln!(
        "[bench] list_characters_brief 平均耗时: {} µs ({:.2} ms) over {} runs, {} files",
        avg_us,
        avg_us as f64 / 1000.0,
        runs,
        d2s_files.len()
    );
}

#[test]
fn bench_d2s_parse_per_file() {
    let dir = fixtures_dir();
    let d2s_files = list_d2s_names(&dir);
    eprintln!("\n[bench] parse_d2s (read_character_info 底层) per file:");

    let mut total_us = 0u128;
    let mut results = Vec::new();

    for d2s_path in &d2s_files {
        let bytes = std::fs::read(d2s_path).expect("read d2s");

        let _ = parse_d2s(&bytes);

        let mut file_total = 0u128;
        let runs = 3;
        for _ in 0..runs {
            let start = Instant::now();
            let result = parse_d2s(&bytes);
            file_total += start.elapsed().as_micros();
            assert!(result.is_ok(), "parse failed: {:?}", result.err());
        }
        let avg_us = file_total / runs as u128;
        total_us += avg_us;
        let file_name = d2s_path.file_name().unwrap().to_string_lossy();
        results.push((file_name.to_string(), avg_us, bytes.len()));
    }

    eprintln!("\n[bench] === parse_d2s 平均耗时 (read_character_info 底层) ===");
    for (name, avg, size) in &results {
        eprintln!(
            "[bench] {:30} avg: {:>6} µs ({:.2} ms) | size: {} bytes",
            name,
            avg,
            *avg as f64 / 1000.0,
            size
        );
    }
    let overall_avg = total_us / results.len() as u128;
    eprintln!(
        "[bench] === 总平均: {} µs ({:.2} ms) per file ===",
        overall_avg,
        overall_avg as f64 / 1000.0
    );
}

#[test]
fn bench_empty_dir() {
    let tmp = std::env::temp_dir().join(format!(
        "d2r_bench_empty_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let start = Instant::now();
    let result = list_characters_brief(tmp.to_string_lossy().to_string());
    let elapsed_us = start.elapsed().as_micros();

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
    eprintln!("\n[bench] list_characters_brief (空目录): {} µs", elapsed_us);
}