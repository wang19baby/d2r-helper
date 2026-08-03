//! Warehouse search tests — unified filter command.
//!
//! Replaces the old per-field list_* commands with a single
//! warehouse_search(filters) endpoint.

use d2r_marketplace_lib::database::{
    Database, WarehousedItem,
    repository::warehouse_repo::WarehouseFilters,
};

fn create_test_db(name: &str) -> Database {
    let tmp = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target").join("tmp").join(format!("test_wh_search_{}_{}.db", std::process::id(), name));
    let _ = std::fs::remove_file(&tmp);
    Database::open(tmp.to_str().unwrap()).expect("create test DB")
}

fn make_item(id: &str, code: &str, kind: &str, quality: Option<&str>,
             slot: Option<&str>, character: Option<&str>) -> WarehousedItem {
    WarehousedItem {
        id: id.to_string(),
        item_code: code.to_string(),
        item_name: format!("Item {}", code),
        item_kind: kind.to_string(),
        quality: quality.map(|s| s.to_string()),
        simple_item: true,
        quantity: 1,
        profile_key: "test".to_string(),
        game_version: String::new(),
        mod_name: String::new(),
        raw_item_bits: vec![],
        raw_bit_length: 0,
        item_json: String::new(),
        stash_name: None,
        imported_at: "2026-07-09T00:00:00Z".to_string(),
        page_name: "默认".to_string(),
        tags: String::new(),
        notes: String::new(),
        source_character: character.map(|s| s.to_string()),
        source_save_path: None,
        slot_equipped: slot.map(|s| s.to_string()),
        page_index: 0,
        position_x: 0,
        position_y: 0,
        inv_width: 1,
        inv_height: 1,
    }
}

#[test]
fn test_warehouse_search_filter_by_character() {
    let db = create_test_db("char_filter");
    let repos = db.repos();

    // Insert test items
    repos.warehouse.add(&make_item("1", "r01", "rune", None, None, Some("EchoingStrike"))).unwrap();
    repos.warehouse.add(&make_item("2", "gcv", "gem", None, None, Some("EchoingStrike"))).unwrap();
    repos.warehouse.add(&make_item("3", "cap", "armor", Some("unique"), Some("helm"), Some("蛮蛮"))).unwrap();

    // Filter by character
    let filters = WarehouseFilters { source_character: Some("EchoingStrike".into()), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 2, "EchoingStrike should have 2 items");
    assert!(results.iter().all(|r| r.item_code == "r01" || r.item_code == "gcv"));
}

#[test]
fn test_warehouse_search_filter_by_slot() {
    let db = create_test_db("slot_filter");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "cap", "armor", Some("unique"), Some("helm"), Some("蛮蛮"))).unwrap();
    repos.warehouse.add(&make_item("2", "amu", "armor", Some("unique"), Some("amulet"), Some("蛮蛮"))).unwrap();
    repos.warehouse.add(&make_item("3", "r01", "rune", None, None, Some("EchoingStrike"))).unwrap();

    let filters = WarehouseFilters { equipment_slot: Some("helm".into()), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 1, "only helm should match");
    assert_eq!(results[0].item_code, "cap");
}

#[test]
fn test_warehouse_search_filter_by_kind() {
    let db = create_test_db("kind_filter");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "r01", "rune", None, None, None)).unwrap();
    repos.warehouse.add(&make_item("2", "r02", "rune", None, None, None)).unwrap();
    repos.warehouse.add(&make_item("3", "gcv", "gem", None, None, None)).unwrap();

    let filters = WarehouseFilters { item_kind: Some("rune".into()), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 2, "2 runes");
}

#[test]
fn test_warehouse_search_filter_by_quality() {
    let db = create_test_db("quality_filter");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "cap", "armor", Some("unique"), None, None)).unwrap();
    repos.warehouse.add(&make_item("2", "r01", "rune", None, None, None)).unwrap();
    repos.warehouse.add(&make_item("3", "xla", "armor", Some("set"), None, None)).unwrap();

    let filters = WarehouseFilters { quality: Some("unique".into()), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 1, "1 unique");
    assert_eq!(results[0].item_code, "cap");
}

#[test]
fn test_warehouse_search_filter_by_text() {
    let db = create_test_db("text_filter");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "r01", "rune", None, None, None)).unwrap();
    repos.warehouse.add(&make_item("2", "gcv", "gem", None, None, None)).unwrap();

    let filters = WarehouseFilters { search_text: Some("r01".into()), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 1, "1 item matching r01");
}

#[test]
fn test_warehouse_search_combined_filters() {
    let db = create_test_db("combined");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "cap", "armor", Some("unique"), Some("helm"), Some("蛮蛮"))).unwrap();
    repos.warehouse.add(&make_item("2", "r01", "rune", None, None, Some("蛮蛮"))).unwrap();
    repos.warehouse.add(&make_item("3", "amu", "armor", Some("unique"), Some("amulet"), Some("EchoingStrike"))).unwrap();

    // Filter: unique + armor
    let filters = WarehouseFilters {
        quality: Some("unique".into()),
        item_kind: Some("armor".into()),
        ..Default::default()
    };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 2, "2 unique armor items");
}

#[test]
fn test_warehouse_search_no_filters_returns_all() {
    let db = create_test_db("all");
    let repos = db.repos();

    repos.warehouse.add(&make_item("1", "r01", "rune", None, None, None)).unwrap();
    repos.warehouse.add(&make_item("2", "gcv", "gem", None, None, None)).unwrap();

    let filters = WarehouseFilters::default();
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 2, "all items returned");
}

#[test]
fn test_warehouse_search_pagination() {
    let db = create_test_db("pagination");
    let repos = db.repos();

    for i in 0..10 {
        let code = format!("r{:02}", i + 1);
        repos.warehouse.add(&make_item(&code, &code, "rune", None, None, None)).unwrap();
    }

    // First 5
    let filters = WarehouseFilters { limit: Some(5), offset: Some(0), ..Default::default() };
    let results = repos.warehouse.search(&filters).expect("search");
    assert_eq!(results.len(), 5, "first 5");
}
