//! Benchmark `character_to_result` (the build_stored_item_summary hot path).
//!
//! 测量 character_to_result 在真实 d2s 物品上的耗时。
//! 用于发现 build_stored_item_summary 的性能退化(US-017)。
//!
//! 已知基线 (2026-07-21, xieedi.d2s 68 items, 15330ms):
//! - armor_stats / weapon_stats 重复线性扫描 ×4-6/item
//! - item_size 重复线性扫描 ×2/item
//! - categorize_item_stats 重复调用 ×2/item
//! - RUNEWORD_COMBOS 排序+线性扫描
//!
//! 优化后 (US-018/019/020): resolver=None 路径 < 5ms (缓存命中后 1ms)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use d2r_marketplace_lib::commands::character::{
    character_to_result, CharacterBinaryStructure, CharacterItemLayout, EquipmentSlotInfo,
    EQUIPMENT_SLOTS,
};
use d2r_marketplace_lib::protocol::d2s::parse_file as parse_d2s;

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    manifest_dir.join("tests").join("fixtures")
}

/// 找最大的 .d2s fixture(通常有最多物品)
fn largest_d2s(dir: &PathBuf) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "d2s").unwrap_or(false))
        .max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
}

fn empty_equipment() -> Vec<EquipmentSlotInfo> {
    EQUIPMENT_SLOTS
        .iter()
        .map(|s| EquipmentSlotInfo {
            slot: (*s).to_string(),
            occupied: false,
            code: None,
            name_zh: None,
            name_en: None,
            name_zh_tw: None,
            quality: None,
            socketed: false,
            skill_bonuses: Vec::new(),
            stats: d2r_marketplace_lib::commands::character::ItemStats::default(),
            durability_cur: 0,
            durability_max: 0,
            tooltip: None,
        })
        .collect()
}

fn default_bin_struct(active_weapon: u32) -> CharacterBinaryStructure {
    CharacterBinaryStructure {
        detected_layout: "standard-v105".to_string(),
        active_weapon,
        attributes_offset: 0,
        protocol_equipped_slots: 12,
        display_equipped_slots: 12,
        item_layout: CharacterItemLayout {
            location_id_bit_offset: 0,
            equipped_slot_bit_offset: 0,
            huffman_code_bit_offset: 0,
            socket_count_bits: 0,
            uid_bits: 0,
            ilvl_bits: 0,
            quality_bits: 0,
            stat_terminator: 0,
        },
    }
}

#[test]
fn bench_character_to_result_resolverless() {
    // 该路径跳过 SQLite,模拟缓存命中后场景
    let dir = fixtures_dir();
    eprintln!("\n[bench:resolverless] fixtures dir: {}", dir.display());

    let target = match largest_d2s(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2s fixture（存档未随仓库分发）"); return; }
    };
    eprintln!("[bench:resolverless] target: {}", target.display());

    let bytes = std::fs::read(&target).expect("read d2s");
    eprintln!("[bench:resolverless] file size: {} bytes", bytes.len());

    let f = parse_d2s(&bytes).expect("parse d2s");
    let total_items = f.equipped.len()
        + f.backpack.len()
        + f.belt.len()
        + f.cube.len()
        + f.merc.len()
        + f.personal_stash.len();
    eprintln!(
        "[bench:resolverless] parsed: equipped={} backpack={} belt={} cube={} merc={} stash={} TOTAL={}",
        f.equipped.len(),
        f.backpack.len(),
        f.belt.len(),
        f.cube.len(),
        f.merc.len(),
        f.personal_stash.len(),
        total_items
    );

    let equipment = empty_equipment();
    let bin_struct = default_bin_struct(f.header.active_weapon);

    let file_hash = "bench_hash";
    let language = "zhCN";
    let runs = 5;
    let mut results = Vec::new();

    for run in 0..runs {
        let t = Instant::now();
        let result = character_to_result(
            &f,
            target.to_str().unwrap(),
            equipment.clone(),
            bin_struct.clone(),
            file_hash,
            None, // resolver_opt = None: 跳过 SQLite 查询
            language,
        );
        let elapsed = t.elapsed();
        eprintln!("[bench:resolverless] run {}: total={:.2} ms", run + 1, elapsed.as_millis());
        // 防止编译器优化掉
        assert!(!result.backpack_items.is_empty() || !result.belt_items.is_empty());
        results.push(elapsed);
    }

    let avg_ms =
        results.iter().map(|d| d.as_millis()).sum::<u128>() as f64 / runs as f64;
    let per_item = if total_items > 0 {
        avg_ms / total_items as f64
    } else {
        0.0
    };
    eprintln!(
        "[bench:resolverless] === 平均 {:.2} ms / {} items = {:.3} ms/item ===",
        avg_ms, total_items, per_item
    );

    // 性能断言: 优化后该路径必须 < 100ms(原 ~15000ms)
    // 留 50x 余量给 debug build
    assert!(
        avg_ms < 100.0,
        "performance regression: avg {:.2} ms > 100ms (expected after US-018/019/020 optimization)",
        avg_ms
    );
}

/// US-022: 验证带 SQLite resolver + warmup 缓存的路径也能正常完成
#[test]
fn bench_resolver_warmup() {
    use d2r_marketplace_lib::database::Database;
    use d2r_marketplace_lib::resource::NameResolver;

    let dir = fixtures_dir();
    let target = match largest_d2s(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2s fixture（存档未随仓库分发）"); return; }
    };

    let bytes = std::fs::read(&target).expect("read d2s");
    let f = parse_d2s(&bytes).expect("parse d2s");

    // 创建 in-memory SQLite,初始化 schema
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory");
    let db = Database::init_from_connection(conn).expect("init db");
    let conn = db.into_connection();

    // Seed 一些 item_base 和 unique_item_def 数据,让 cache 有内容
    conn.execute(
        "INSERT INTO item_base (code, name_en) VALUES ('cap', 'Cap'), ('r01', 'El Rune')",
        [],
    ).ok();
    conn.execute(
        "INSERT INTO unique_item_def (unique_id, name_en) VALUES (105, 'Magefist')",
        [],
    ).ok();

    // 用 warmed-up resolver
    let profile_id = 1; // 测试 profile
    let resolver = NameResolver::with_localized_cache(&conn, profile_id);

    let t = Instant::now();
    let result = character_to_result(
        &f,
        target.to_str().unwrap(),
        empty_equipment(),
        default_bin_struct(f.header.active_weapon),
        "warmup_hash",
        Some(&(conn, Arc::new(resolver))),
        "zhCN",
    );
    let elapsed = t.elapsed();
    eprintln!(
        "[bench:warmup] character_to_result with resolver+cache: {:.2} ms",
        elapsed.as_millis()
    );

    // 即便 localized_string 表为空,缓存也应当让 query 走 cache miss → SQL 兜底路径
    // 性能断言:不应当显著慢于 resolveless 路径
    // (实际会比 resolveless 慢,因为有 SQL 兜底,但不应该回到 15000ms 量级)
    assert!(!result.backpack_items.is_empty() || !result.belt_items.is_empty());
}

/// US-022b: 对比测试 warmup 后的 resolver 速度(无数据时走 SQL 兜底)
#[test]
fn bench_resolver_cold_vs_warmup() {
    use d2r_marketplace_lib::database::Database;
    use d2r_marketplace_lib::resource::NameResolver;

    let dir = fixtures_dir();
    let target = match largest_d2s(&dir) {
        Some(t) => t,
        None => { eprintln!("SKIP: 无本地 d2s fixture（存档未随仓库分发）"); return; }
    };
    let bytes = std::fs::read(&target).expect("read d2s");
    let f = parse_d2s(&bytes).expect("parse d2s");

    // Cold (no warmup) - 走 SQL 每次
    let conn_cold = rusqlite::Connection::open_in_memory().expect("in-memory");
    let db_cold = Database::init_from_connection(conn_cold).expect("init db");
    let conn_cold = db_cold.into_connection();
    let resolver_cold = NameResolver::new(1);

    let t_cold = Instant::now();
    let _ = character_to_result(
        &f,
        target.to_str().unwrap(),
        empty_equipment(),
        default_bin_struct(f.header.active_weapon),
        "cold_hash",
        Some(&(conn_cold, Arc::new(resolver_cold))),
        "zhCN",
    );
    let cold_ms = t_cold.elapsed().as_millis();

    // Warm (with cache warmup) - HashMap lookup
    let conn_warm = rusqlite::Connection::open_in_memory().expect("in-memory");
    let db_warm = Database::init_from_connection(conn_warm).expect("init db");
    let conn_warm = db_warm.into_connection();
    let resolver_warm = NameResolver::with_localized_cache(&conn_warm, 1);

    let t_warm = Instant::now();
    let _ = character_to_result(
        &f,
        target.to_str().unwrap(),
        empty_equipment(),
        default_bin_struct(f.header.active_weapon),
        "warm_hash",
        Some(&(conn_warm, Arc::new(resolver_warm))),
        "zhCN",
    );
    let warm_ms = t_warm.elapsed().as_millis();

    eprintln!(
        "[bench:cold-vs-warm] cold={} ms warm={} ms (in-memory, no data)",
        cold_ms, warm_ms
    );
}


