/*!
 * warehouse_deposit / warehouse_withdraw 端到端测试
 *
 * T01-T06: Happy path (deposit / withdraw / round-trip)
 * T16:     Out of bounds withdraw
 * T17:     Position collision + double withdraw
 * T18:     Stackable merge (withdraw to page with same rune)
 *
 * 设计:
 * - 直接调用 inner functions (warehouse_deposit_inner / warehouse_withdraw_inner),
 *   绕过 tauri::State 包装
 * - 手工 seed warehouse DB (用 db.warehouse_add),不依赖 deposit 命令
 * - 因为 Database 是 owned,seed 必须在 build_app_state 之前
 */

use d2r_marketplace_lib::commands::warehouse::{check_withdraw_position, warehouse_deposit_inner, warehouse_withdraw_inner};
use d2r_marketplace_lib::AppState;
use rusqlite::params;
use d2r_marketplace_lib::database::{Database, WarehousedItem};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::{Arc, Mutex};

// ── Helpers ──

fn nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn copy_fixture_to_temp(name: &str) -> Option<std::path::PathBuf> {
    let src = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return None };
    let dst = std::env::temp_dir()
        .join(format!("d2r_e2e_{}_{}_{}.d2i", std::process::id(), name, nanos()));
    let _ = std::fs::remove_file(&dst);
    std::fs::copy(&src, &dst).expect("copy fixture");
    Some(dst)
}

fn create_test_db(name: &str) -> Database {
    let tmp = std::env::temp_dir().join(format!("d2r_e2e_{}_{}_{}.db", std::process::id(), name, nanos()));
    let _ = std::fs::remove_file(&tmp);
    Database::open(tmp.to_str().unwrap()).expect("create test db")
}

/// Seed warehouse DB with a rune. Then move the Database into AppState.
fn seed_rune_in_db(db: &Database, inv_w: u8, inv_h: u8) -> String {
    let id = format!("test-rune-{}", nanos());
    let raw_bits = vec![0xAA, 0xBB, 0xCC];
    let item = WarehousedItem {
        id: id.clone(),
        item_code: "r19".to_string(),
        item_name: "El Rune (test seed)".to_string(),
        item_kind: "rune".to_string(),
        quality: Some("normal".to_string()),
        simple_item: true,
        quantity: 1,
        profile_key: "mod:vanilla:2.7".to_string(),
        game_version: "2.7".to_string(),
        mod_name: String::new(),
        raw_item_bits: raw_bits,
        raw_bit_length: 24,
        item_json: serde_json::json!({
            "item_type": "r19",
            "amount": 1,
            "inv_width": inv_w,
            "inv_height": inv_h,
            "position_x": 0,
            "position_y": 0,
        }).to_string(),
        stash_name: Some("test.d2i".to_string()),
        imported_at: "2026-01-01T00:00:00Z".to_string(),
        page_name: "默认收藏".to_string(),
        tags: String::new(),
        notes: String::new(),
        source_character: None,
        source_save_path: None,
        slot_equipped: None,
        page_index: 0,
        position_x: 0,
        position_y: 0,
        inv_width: inv_w as i32,
        inv_height: inv_h as i32,
    };
    db.warehouse_add(&item).expect("warehouse_add");
    id
}

fn sha256_of(path: &std::path::Path) -> String {
    let mut f = std::fs::File::open(path).expect("open");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("read");
    let mut hasher = Sha256::new();
    hasher.update(&buf);
    format!("{:x}", hasher.finalize())
}

/// Configure test DB so get_active_profile_key returns "mod:vanilla:2.7"
/// matching the seeded warehoused item profile_key.
fn setup_active_profile(db: &Database) {
    let conn = db.get_connection();
    // Insert config rows used by get_active_profile_key
    // build_profile_key("", "2.7") returns "vanilla:2.7"
    // build_profile_key("vanilla", "2.7") returns "mod:vanilla:2.7"
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
        params!["active_mod", "vanilla"],
    ).unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
        params!["game_version", "2.7"],
    ).unwrap();
    // Insert matching resource_profile row so get_active_profile_id returns non-zero
    conn.execute(
        "INSERT OR IGNORE INTO resource_profile (profile_key, source_kind, mod_name, game_version, active_language, game_root, excel_path, checksum, source_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            "mod:vanilla:2.7",
            "vanilla",
            "vanilla",
            "2.7",
            "enUS",
            "",
            "",
            "",
            "",
            "2026-01-01T00:00:00Z",
        ],
    ).unwrap();
}
/// Find any item matching `code` in the fixture across all pages.
/// Returns (page_index, x, y, amount). The page_index matches the
/// parser::ParsedItem::page_index used by warehouse_deposit_inner.
fn find_fixture_item(code: &str) -> Option<(usize, u8, u8, u32)> {
    use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
    use d2r_marketplace_lib::protocol::d2i::legacy::item::read_all_stash_items;
    let src = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return None };
    let data = std::fs::read(&src).expect("read fixture");
    let pages_info = split_legacy_d2i_pages(&data).expect("parse fixture");
    let all_items = read_all_stash_items(&pages_info.pages).expect("read items");
    for (page_idx, items) in &all_items {
        for item in items {
            if item.item_type == code && item.amount > 0 {
                return Some((*page_idx, item.position_x, item.position_y, item.amount as u32));
            }
        }
    }
    panic!("fixture does not contain item code '{}'", code);
}

/// Build a test harness: copy fixture to temp, create DB + AppState, setup profile.
struct DepositTestHarness {
    pub stash_path: std::path::PathBuf,
    pub state: AppState,
}

fn setup_harness(name: &str) -> Option<DepositTestHarness> {
    let Some(stash_path) = copy_fixture_to_temp(name) else { return None };
    let db = create_test_db(&format!("{}_db", name));
    setup_active_profile(&db);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };
    Some(DepositTestHarness { stash_path, state })
}

// ── T01: deposit full removal stackable ──
// Deposit a full stack of a rune from the fixture. Verify d2i reduces item count
// and DB records it.

/// fixture 可能不随仓库分发（用户本地存档）——缺失时 SKIP。
fn fixture_path(name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures").join(name);
    if p.exists() {
        Some(p)
    } else {
        eprintln!("SKIP: fixture {} 未随仓库分发（本地存档）, 跳过测试", name);
        None
    }
}

#[test]
fn test_deposit_full_removal_stackable() {
    let Some(harness) = setup_harness("t01") else { return };
    let stash_str = harness.stash_path.to_string_lossy().to_string();

    // Find a known rune on page[1]: "r19" (El Rune)
    let Some((page_idx, px, py, available)) = find_fixture_item("r19") else { return };
    assert!(available > 0, "fixture must have at least 1 r19");

    let items_before = {
        let db = harness.state.db.lock().unwrap();
        db.warehouse_list_all().unwrap().len()
    };
    let d2i_before = sha256_of(&harness.stash_path);

    let result = warehouse_deposit_inner(
        &harness.state, stash_str, "r19".to_string(), page_idx, px, py,
        available, Some("测试收藏".to_string()),
    );
    assert!(result.is_ok(), "deposit full stack must succeed (got: {:?})", result);

    // DB must have one more record
    let items_after = {
        let db = harness.state.db.lock().unwrap();
        let all = db.warehouse_list_all().unwrap();
        assert_eq!(all.len(), items_before + 1, "DB must have 1 new record");
        let deposited = all.iter().find(|i| i.item_code == "r19").expect("r19 must be in DB");
        assert_eq!(deposited.quantity, available, "quantity must match");
        assert!(!deposited.raw_item_bits.is_empty(), "raw_item_bits must be non-empty");
        all.len()
    };

    // d2i must have changed (r19 removed)
    let d2i_after = sha256_of(&harness.stash_path);
    assert_ne!(d2i_before, d2i_after, "d2i must change after deposit");

    // Cleanup
    let _ = std::fs::remove_file(&harness.stash_path);
    drop(harness);
}

// ── T02: deposit partial qty stackable ──
// Deposit only part of a rune stack, verify stash retains remainder.

#[test]
fn test_deposit_partial_qty_stackable() {
    let Some(harness) = setup_harness("t02") else { return };
    let stash_str = harness.stash_path.to_string_lossy().to_string();

    let Some((page_idx, px, py, available)) = find_fixture_item("r19") else { return };
    assert!(available >= 1, "fixture must have r19");

    let deposit_qty = 1;
    let d2i_before = sha256_of(&harness.stash_path);
    let db_before = harness.state.db.lock().unwrap().warehouse_list_all().unwrap().len();

    let result = warehouse_deposit_inner(
        &harness.state, stash_str, "r19".to_string(), page_idx, px, py,
        deposit_qty, None,
    );
    assert!(result.is_ok(), "partial deposit must succeed (got: {:?})", result);

    // DB must have a new record
    let db_after = harness.state.db.lock().unwrap().warehouse_list_all().unwrap().len();
    assert_eq!(db_after, db_before + 1, "DB must have 1 new record");
    let deposited = harness.state.db.lock().unwrap().warehouse_list_all().unwrap()
        .into_iter().find(|i| i.item_code == "r19")
        .expect("r19 must be in DB after deposit");
    assert_eq!(deposited.quantity, deposit_qty, "deposited qty must match");

    // d2i must have changed
    let d2i_after = sha256_of(&harness.stash_path);
    assert_ne!(d2i_before, d2i_after, "d2i must change after deposit");
}

// ── T03: withdraw to empty cell ──
// Seed a warehouse record, then withdraw it to a known-empty cell on page[1].
// Verify d2i gains an item and DB record is removed.

#[test]
fn test_withdraw_to_empty_cell() {
    let Some(stash_path) = copy_fixture_to_temp("t03") else { return };
    let stash_str = stash_path.to_string_lossy().to_string();

    // Find an empty cell on page[1]
    let stash_data = std::fs::read(&stash_path).expect("read d2i");
    let (empty_page, empty_x, empty_y) = (1..16)
        .flat_map(|p| (0..16u8).flat_map(move |y| (0..16u8).map(move |x| (p, x, y))))
        .find(|(p, x, y)| check_withdraw_position(&stash_data, *p, *x, *y, 1, 1).is_ok())
        .expect("fixture must have at least one empty cell");
    drop(stash_data);

    let db = create_test_db("t03_db");
    setup_active_profile(&db);
    let rune_id = seed_rune_in_db(&db, 1, 1);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };

    let d2i_before = sha256_of(&stash_path);
    let db_count_before = state.db.lock().unwrap().warehouse_list_all().unwrap().len();

    let result = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), empty_page, empty_x, empty_y, None,
    );
    assert!(result.is_ok(), "withdraw to empty cell must succeed (got: {:?})", result);

    // DB record must be removed
    let db_count_after = state.db.lock().unwrap().warehouse_list_all().unwrap().len();
    assert_eq!(db_count_after, db_count_before - 1, "DB must have 1 fewer record");

    // d2i must have changed (new item added)
    let d2i_after = sha256_of(&stash_path);
    assert_ne!(d2i_before, d2i_after, "d2i must change after withdraw");

    // Auto-backup should exist
    let auto_dir = std::path::Path::new(&stash_path).parent()
        .unwrap().join("marketplace_backups").join("auto");
    assert!(auto_dir.exists(), "auto-backup dir must exist");
    let backup_count = std::fs::read_dir(&auto_dir)
        .map(|e| e.count())
        .unwrap_or(0);
    assert!(backup_count > 0, "at least 1 auto-backup must exist");

    let _ = std::fs::remove_file(&stash_path);
    drop(state);
}

// ── T04: withdraw preserves other items ──
// Withdraw one, verify remaining items' bytes are intact (by re-parsing).

#[test]
fn test_withdraw_preserves_other_items() {
    let Some(stash_path) = copy_fixture_to_temp("t04") else { return };
    let stash_str = stash_path.to_string_lossy().to_string();

    // Count items before
    let data_before = std::fs::read(&stash_path).expect("read d2i");
    let sha_before = sha256_of(&stash_path);

    let (empty_page, empty_x, empty_y) = (1..16)
        .flat_map(|p| (0..16u8).flat_map(move |y| (0..16u8).map(move |x| (p, x, y))))
        .find(|(p, x, y)| check_withdraw_position(&data_before, *p, *x, *y, 1, 1).is_ok())
        .expect("fixture must have at least one empty cell");
    drop(data_before);

    let db = create_test_db("t04_db");
    setup_active_profile(&db);
    let rune_id = seed_rune_in_db(&db, 1, 1);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };

    let result = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), empty_page, empty_x, empty_y, None,
    );
    assert!(result.is_ok(), "withdraw must succeed");

    // Re-parse the stash — other items must still parse correctly
    let data_after = std::fs::read(&stash_path).expect("read d2i after");
    use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
    use d2r_marketplace_lib::protocol::d2i::legacy::item::read_all_stash_items;
    let pages_info = split_legacy_d2i_pages(&data_after).expect("parse after");
    let items_result = read_all_stash_items(&pages_info.pages);
    assert!(items_result.is_ok(), "re-parse must succeed after withdraw");

    // SHA must be different (stash was modified)
    let sha_after = sha256_of(&stash_path);
    assert_ne!(sha_before, sha_after, "stash must differ after withdraw");

    let _ = std::fs::remove_file(&stash_path);
    drop(state);
}

// ── T05: round-trip raw bits ──
// Deposit a rune, withdraw it back. Verify DB and stash change as expected.

#[test]
fn test_round_trip_raw_bits() {
    let Some(harness) = setup_harness("t05") else { return };
    let stash_str = harness.stash_path.to_string_lossy().to_string();

    let Some((page_idx, px, py, available)) = find_fixture_item("r19") else { return };
    assert!(available > 0, "fixture must have r19");

    let deposit_qty = if available > 1 { 1 } else { available };

    // Deposit
    let r1 = warehouse_deposit_inner(
        &harness.state, stash_str.clone(), "r19".to_string(), page_idx, px, py,
        deposit_qty, None,
    );
    assert!(r1.is_ok(), "deposit must succeed (got: {:?})", r1);

    let item_id = {
        let db = harness.state.db.lock().unwrap();
        db.warehouse_list_all().unwrap()
            .into_iter().find(|i| i.item_code == "r19")
            .expect("r19 in DB after deposit").id
    };

    // Find empty cell for withdraw
    let stash_data = std::fs::read(&harness.stash_path).expect("read");
    let (wp, wx, wy) = (1..16)
        .flat_map(|p| (0..16u8).flat_map(move |y| (0..16u8).map(move |x| (p, x, y))))
        .find(|(p, x, y)| check_withdraw_position(&stash_data, *p, *x, *y, 1, 1).is_ok())
        .expect("empty cell for withdraw");
    drop(stash_data);

    // Withdraw
    let r2 = warehouse_withdraw_inner(
        &harness.state, item_id.clone(), stash_str.clone(), wp, wx, wy, None,
    );
    assert!(r2.is_ok(), "withdraw must succeed (got: {:?})", r2);

    // DB record must be removed
    let records_after = harness.state.db.lock().unwrap().warehouse_list_all().unwrap();
    assert!(!records_after.iter().any(|i| i.id == item_id), "warehouse record must be removed after withdraw");
}

// ── T06: round-trip with different position ──
// Deposit a rune, withdraw it to a different position. Verify DB record removed
// and stash file changed.

#[test]
fn test_round_trip_with_different_position() {
    let Some(harness) = setup_harness("t06") else { return };
    let stash_str = harness.stash_path.to_string_lossy().to_string();

    let Some((page_idx, px, py, available)) = find_fixture_item("r19") else { return };
    assert!(available > 0, "fixture must have r19");

    let deposit_qty = if available > 1 { 1 } else { available };

    // Deposit
    let r1 = warehouse_deposit_inner(
        &harness.state, stash_str.clone(), "r19".to_string(), page_idx, px, py,
        deposit_qty, None,
    );
    assert!(r1.is_ok(), "deposit must succeed");

    let item_id = {
        let db = harness.state.db.lock().unwrap();
        db.warehouse_list_all().unwrap()
            .into_iter().find(|i| i.item_code == "r19")
            .expect("r19 in DB").id
    };

    let d2i_before = sha256_of(&harness.stash_path);
    let db_count_before = harness.state.db.lock().unwrap().warehouse_list_all().unwrap().len();

    // Find an empty cell and withdraw
    let stash_data = std::fs::read(&harness.stash_path).expect("read");
    let (wp, wx, wy) = (1..16)
        .flat_map(|p| (10..16u8).flat_map(move |y| (10..16u8).map(move |x| (p, x, y))))
        .find(|(p, x, y)| check_withdraw_position(&stash_data, *p, *x, *y, 1, 1).is_ok())
        .expect("empty cell at (10,10) area");
    drop(stash_data);

    eprintln!("T06: withdrawing to (page={}, x={}, y={})", wp, wx, wy);

    let r2 = warehouse_withdraw_inner(
        &harness.state, item_id, stash_str.clone(), wp, wx, wy, None,
    );
    assert!(r2.is_ok(), "withdraw to ({},{}) must succeed (got: {:?})", wx, wy, r2);

    // DB record must be removed
    let db_count_after = harness.state.db.lock().unwrap().warehouse_list_all().unwrap().len();
    assert_eq!(db_count_after, db_count_before - 1, "DB must have 1 fewer record");

    // d2i must have changed (item was placed)
    let d2i_after = sha256_of(&harness.stash_path);
    assert_ne!(d2i_before, d2i_after, "d2i must change after withdraw");

    let _ = std::fs::remove_file(&harness.stash_path);
    drop(harness);
}

// ── T16: out of bounds ──

#[test]
fn test_withdraw_out_of_bounds_returns_err() {
    let Some(stash_path) = copy_fixture_to_temp("t16_oob") else { return };
    let stash_str = stash_path.to_string_lossy().to_string();

    let db = create_test_db("t16_oob_db");
    setup_active_profile(&db);
    let rune_id = seed_rune_in_db(&db, 1, 1);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };

    let d2i_before = sha256_of(&stash_path);

    let result = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), 0, 200, 200, None,
    );

    assert!(result.is_err(), "withdraw to (200,200) must return Err");
    let err = result.unwrap_err();
    let err_lower = err.to_lowercase();
    let is_bounds_err = err_lower.contains("out of bounds")
        || err_lower.contains("exceed")
        || err_lower.contains("invalid position")
        || err_lower.contains("position")
        || err_lower.contains("invalid");
    assert!(is_bounds_err, "expected bounds error, got: {}", err);

    let d2i_after = sha256_of(&stash_path);
    assert_eq!(d2i_before, d2i_after, "OOB withdraw must not modify d2i");
    let records_after = state.db.lock().unwrap().warehouse_list_all().unwrap();
    assert_eq!(records_after.len(), 1, "OOB withdraw must not delete record");
    assert!(records_after.iter().any(|r| r.id == rune_id), "rune record must still exist");
}

// ── T17a: position collision (red — 修复前 fail) ──

#[test]
fn test_withdraw_to_occupied_position() {
    // 解析真实 fixture 找 page[0] (装备页) 中一个已知 (x, y) 占位
    // 然后尝试把 seed 的 1x1 item 放到该坐标,期望 Err "occupied"
    use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
    use d2r_marketplace_lib::protocol::d2i::legacy::item::read_all_stash_items;

    let Some(stash_path) = copy_fixture_to_temp("t17_collision") else { return };
    let stash_str = stash_path.to_string_lossy().to_string();

    // 找 page[0] 第一个有 (x, y) 的 item
    let data = std::fs::read(&stash_path).expect("read d2i");
    let pages_info = split_legacy_d2i_pages(&data).expect("parse d2i");
    let all_items = read_all_stash_items(&pages_info.pages).expect("read items");
    let page0_items: Vec<_> = all_items.iter()
        .find(|(idx, _)| *idx == 0)
        .map(|(_, items)| items.clone())
        .unwrap_or_default();
    assert!(!page0_items.is_empty(), "fixture page[0] must have at least 1 item");
    let occupied = &page0_items[0];
    let occ_x = occupied.position_x;
    let occ_y = occupied.position_y;
    eprintln!("T17: target occupied cell = ({}, {}), item = {}", occ_x, occ_y, occupied.item_type);

    // Setup DB + seed
    let db = create_test_db("t17_collision_db");
    setup_active_profile(&db);
    let rune_id = seed_rune_in_db(&db, 1, 1);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };

    let d2i_before = sha256_of(&stash_path);
    let records_before = state.db.lock().unwrap().warehouse_list_all().unwrap();
    assert_eq!(records_before.len(), 1);

    // Try to withdraw to the occupied (occ_x, occ_y)
    let result = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), 0, occ_x, occ_y, None,
    );

    // 修复后期望: Err "occupied" / "collision"
    // 当前 (red): 可能 Err (Windows mmap write 失败) 或 Ok (覆盖)
    let err = result.expect_err("withdraw to occupied cell must return Err");
    let err_lower = err.to_lowercase();
    let is_collision_err = err_lower.contains("occupied")
        || err_lower.contains("collision")
        || err_lower.contains("occupied by");
    assert!(is_collision_err,
        "expected 'occupied' / 'collision' error after fix, got: {}", err);

    // 冲突时: d2i 不变, db record 仍存在
    let d2i_after = sha256_of(&stash_path);
    assert_eq!(d2i_before, d2i_after, "collision reject must not modify d2i");
    let records_after = state.db.lock().unwrap().warehouse_list_all().unwrap();
    assert_eq!(records_after.len(), 1, "collision reject must not delete record");
    assert!(records_after.iter().any(|r| r.id == rune_id));
}

// ── T17b: double withdraw same id (baseline, 已工作的) ──

#[test]
fn test_withdraw_double_withdraw_same_id() {
    let Some(stash_path) = copy_fixture_to_temp("t17_double") else { return };
    let stash_str = stash_path.to_string_lossy().to_string();

    // Find a cell that the implementation's bounds+collision check accepts,
    // so the first withdraw can actually succeed (regression coverage).
    // Use a stackable page (page 1) where 1x1 items rarely fill the grid.
    let stash_data = std::fs::read(&stash_path).expect("read d2i");
    let (empty_page, empty_x, empty_y) = (1..16)
        .flat_map(|p| (0..16u8).flat_map(move |y| (0..16u8).map(move |x| (p, x, y))))
        .find(|(p, x, y)| check_withdraw_position(&stash_data, *p, *x, *y, 1, 1).is_ok())
        .expect("fixture must have at least one empty cell across pages 1-15");
    eprintln!("T17b: using empty cell (page={}, x={}, y={})", empty_page, empty_x, empty_y);

    let db = create_test_db("t17_double_db");
    setup_active_profile(&db);
    let rune_id = seed_rune_in_db(&db, 1, 1);
    let state = AppState {
        db: Mutex::new(db),
        import_state: Arc::new(Mutex::new(d2r_marketplace_lib::ImportState::new())),
    };

    // First withdraw: must pass bounds+collision AND successfully write
    // the stash (Windows mmap + write conflict fixed in Sprint 2 W5).
    let first = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), empty_page, empty_x, empty_y, None,
    );
    assert!(first.is_ok(), "first withdraw must succeed (got: {:?})", first);
    let d2i_after_first = sha256_of(&stash_path);
    assert_ne!(d2i_after_first.len(), 0, "d2i must exist after first withdraw");

    // 第二次 withdraw 同一 id → 必须 Err (record already removed)
    let second = warehouse_withdraw_inner(
        &state, rune_id.clone(), stash_str.clone(), empty_page, empty_x, empty_y, None,
    );
    assert!(second.is_err(), "second withdraw of same id must return Err");

    let d2i_after_second = sha256_of(&stash_path);
    assert_eq!(d2i_after_first, d2i_after_second,
        "failed second withdraw must not modify d2i");
}

// ── T18: withdraw to stackable page merges (regression: 
// was `is_mod_stash && page.is_stackable`, skipped non-mod).

#[test]
fn test_withdraw_to_stackable_page_merges() {
    let Some(harness) = setup_harness("t18") else { return };
    let stash_str = harness.stash_path.to_string_lossy().to_string();

    let stash_data = std::fs::read(&harness.stash_path).expect("read");
    use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
    let file = parse_file(&stash_data).expect("parse");
    let page1 = file.pages.get(1).expect("page[1]");
    assert!(page1.is_stackable, "page[1] must be stackable");
    let page1_items: Vec<_> = file.items.iter()
        .filter(|pi| pi.page_index == 1 && pi.item.amount > 0)
        .collect();
    let r19_page1 = page1_items.iter()
        .find(|pi| pi.item.code == "r19")
        .expect("page[1] must have r19");
    let initial_amount = r19_page1.item.amount;
    let page1_idx = 1;
    drop(stash_data);

    let deposit_qty = initial_amount.saturating_sub(1);
    assert!(deposit_qty > 0);
    let r1 = warehouse_deposit_inner(
        &harness.state, stash_str.clone(), "r19".to_string(), page1_idx,
        r19_page1.item.x, r19_page1.item.y, deposit_qty, None,
    );
    assert!(r1.is_ok(), "deposit must succeed (got: {:?})", r1);

    let item_id = {
        let db = harness.state.db.lock().unwrap();
        let all = db.warehouse_list_all().unwrap();
        all.iter().find(|i| i.item_code == "r19")
            .expect("r19 in DB").id.clone()
    };

    let stash_after_dep = std::fs::read(&harness.stash_path).expect("read");
    let file2 = parse_file(&stash_after_dep).expect("parse");
    let remain: Vec<_> = file2.items.iter()
        .filter(|pi| pi.page_index == page1_idx && pi.item.code == "r19" && pi.item.amount > 0)
        .collect();
    assert_eq!(remain.len(), 1, "must be exactly 1 r19 after partial deposit");
    assert_eq!(remain[0].item.amount, 1, "must have 1 r19 remaining");
    drop(stash_after_dep);
    drop(file2);

    let r2 = warehouse_withdraw_inner(
        &harness.state, item_id.clone(), stash_str.clone(),
        page1_idx, 0, 0, Some(1),
    );
    assert!(r2.is_ok(), "withdraw must succeed (got: {:?})", r2);

    let stash_after_wd = std::fs::read(&harness.stash_path).expect("read");
    let file3 = parse_file(&stash_after_wd).expect("parse");
    let final_r19: Vec<_> = file3.items.iter()
        .filter(|pi| pi.page_index == page1_idx && pi.item.code == "r19" && pi.item.amount > 0)
        .collect();
    assert_eq!(final_r19.len(), 1,
        "MERGING FAILED: {} r19 entries, expected 1 (regression of is_mod_stash bug)",
        final_r19.len());
    assert_eq!(final_r19[0].item.amount, initial_amount,
        "amount must be restored to {} after merge, got {}",
        initial_amount, final_r19[0].item.amount);

    let records = harness.state.db.lock().unwrap().warehouse_list_all().unwrap();
    assert!(!records.iter().any(|r| r.id == item_id),
        "warehouse record must be removed");

    let _ = std::fs::remove_file(&harness.stash_path);
    drop(harness);
}
