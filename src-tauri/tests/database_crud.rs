//! Database SQLite CRUD tests
//!
//! 用 temp :memory: SQLite 数据库测试所有 Database API:
//! - User / Balance
//! - VirtualItem 增删改查
//! - ListedItem / mark_listing_*
//! - Transaction 添加
//! - process_due_listings (auto-sell)
//! - AppConfig get/set
//! - Warehouse CRUD (add, list, get, remove, update_meta)
//!
//! 用 Database::open(":memory:") 临时数据库,test 结束后自动释放。

use d2r_marketplace_lib::database::db::Database;
use d2r_marketplace_lib::database::models::WarehousedItem;
use d2r_marketplace_lib::database::models::VirtualItem;
use d2r_marketplace_lib::protocol::d2i::legacy::resource_manifest::{ResourceFileInfo, ResourceManifest};

fn make_db() -> Database {
    Database::open(":memory:").expect("open :memory: db")
}

fn make_item(id: &str, code: &str, qty: i64) -> VirtualItem {
    VirtualItem {
        id: id.into(),
        name: format!("Test {code}"),
        item_code: Some(code.into()),
        item_kind: Some("rune".into()),
        item_type: Some(code.to_uppercase()),
        quality: Some("Normal".into()),
        level: Some(1),
        attributes: Some("".into()),
        source: Some("test".into()),
        exported_from: None,
        purchased_at: None,
        token_price: Some(6900),
        status: Some("listed".into()),
        quantity: Some(qty),
        unit_price: Some(6900),
        listed_at: Some("2026-07-05T10:00:00Z".into()),
        sell_after_seconds: Some(3600),
        profile_id: Some(1),
        profile_key: Some("vanilla:default".into()),
        game_version: Some(String::new()),
        mod_name: Some(String::new()),
    }
}

// ═══════ User / Balance ═══════

#[test]
fn test_initial_seed_user_balance_10000() {
    let db = make_db();
    let bal = db.get_token_balance().expect("get_balance");
    assert_eq!(bal, 10000, "seed user balance should be 10000");
}

#[test]
fn test_token_balance_increment() {
    let db = make_db();
    db.update_token_balance(500).expect("inc");
    assert_eq!(db.get_token_balance().unwrap(), 10500);
}

#[test]
fn test_token_balance_decrement() {
    let db = make_db();
    db.update_token_balance(-3000).expect("dec");
    assert_eq!(db.get_token_balance().unwrap(), 7000);
}

// ═══════ Virtual Items ═══════

#[test]
fn test_add_and_get_virtual_item() {
    let db = make_db();
    let item = make_item("v-001", "r01", 1);
    db.add_virtual_item(&item).expect("add");
    let got = db.get_virtual_item_by_id("v-001").expect("get");
    assert!(got.is_some(), "item should be found");
    let got = got.unwrap();
    assert_eq!(got.item_code, Some("r01".into()));
    assert_eq!(got.unit_price, Some(6900));
}

#[test]
fn test_get_nonexistent_returns_none() {
    let db = make_db();
    let got = db.get_virtual_item_by_id("v-missing");
    assert!(got.is_ok());
    assert!(got.unwrap().is_none());
}

#[test]
fn test_virtual_items_filter_by_status() {
    let db = make_db();
    // items 默认 status = "listed"
    db.add_virtual_item(&make_item("v-001", "r01", 1)).ok();
    db.add_virtual_item(&make_item("v-002", "r02", 1)).ok();
    db.add_virtual_item(&make_item("v-003", "r03", 1)).ok();
    db.mark_item_as_sold("v-002", "vanilla:default").ok();

    // get_virtual_items 用 EXACT status match
    let listed = db.get_virtual_items("listed").expect("listed");
    assert_eq!(listed.len(), 2, "v-001 + v-003 are 'listed', v-002 is 'sold'");

    let sold = db.get_virtual_items("sold").expect("sold");
    assert_eq!(sold.len(), 1, "v-002 is 'sold'");

    let available = db.get_virtual_items("available").expect("available");
    assert_eq!(available.len(), 0, "no item has 'available' status");

    // get_listed_items 是高级封装,内部 WHERE status='listed'
    let via_helper = db.get_listed_items().expect("via helper");
    assert_eq!(via_helper.len(), 2, "get_listed_items helper also filters 'listed'");
}

#[test]
fn test_virtual_items_filter_by_profile_key() {
    let db = make_db();
    let vanilla = make_item("v-001", "r01", 1);
    let mut modded = make_item("v-002", "r02", 1);
    modded.profile_key = Some("mod:testmod:3.2".into());

    db.add_virtual_item(&vanilla).expect("add vanilla");
    db.add_virtual_item(&modded).expect("add modded");

    let listed = db
        .get_listed_items_in_profile("vanilla:default")
        .expect("listed in profile");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "v-001");

    let available = db
        .get_virtual_items_in_profile("listed", "mod:testmod:3.2")
        .expect("virtual items in profile");
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id, "v-002");
}

#[test]
fn test_mark_listing_cancelled() {
    let db = make_db();
    db.add_virtual_item(&make_item("v-001", "r01", 1)).ok();
    let cancelled = db.mark_listing_cancelled("v-001", "vanilla:default").expect("cancel");
    assert!(cancelled, "should return true (affected > 0)");

    // Cancel again returns false (status not 'listed' anymore)
    let second = db.mark_listing_cancelled("v-001", "vanilla:default").expect("cancel 2");
    assert!(!second, "second cancel should return false");
}

#[test]
fn test_mark_listing_sold() {
    let db = make_db();
    db.add_virtual_item(&make_item("v-001", "r01", 1)).ok();
    let sold = db.mark_listing_sold("v-001").expect("sell");
    assert!(sold);

    let got = db.get_virtual_item_by_id("v-001").unwrap().unwrap();
    assert_eq!(got.status, Some("sold".into()));
    assert!(got.purchased_at.is_some(), "purchased_at should be set");
}

#[test]
fn test_mark_item_as_imported() {
    let db = make_db();
    db.add_virtual_item(&make_item("v-001", "r01", 1)).ok();
    db.mark_item_as_imported("v-001", "vanilla:default").expect("import");
    let got = db.get_virtual_item_by_id("v-001").unwrap().unwrap();
    assert_eq!(got.status, Some("imported".into()));
}

// ═══════ Transactions ═══════

#[test]
fn test_add_transaction() {
    let db = make_db();
    // Need to add the item first — transaction has FOREIGN KEY on item_id
    db.add_virtual_item(&make_item("v-001", "r01", 1)).ok();
    db.add_transaction("purchase", Some("v-001"), 6900, "Bought El Rune")
        .expect("add tx");
    // No direct getter, but no panic = pass
}

#[test]
fn test_add_transaction_without_item_id() {
    // item_id can be None (e.g., daily bonus)
    let db = make_db();
    db.add_transaction("bonus", None, 1000, "daily bonus")
        .expect("add tx with None");
}

// ═══════ App Config ═══════

#[test]
fn test_config_get_set() {
    let db = make_db();
    // Initially not set
    let v0 = db.get_config("save_folder").expect("get");
    assert!(v0.is_none());

    db.set_config("save_folder", "D:/saves/").expect("set");
    let v = db.get_config("save_folder").expect("get 2");
    assert_eq!(v, Some("D:/saves/".into()));

    // Overwrite
    db.set_config("save_folder", "E:/new/").expect("set 2");
    let v = db.get_config("save_folder").unwrap();
    assert_eq!(v, Some("E:/new/".into()));
}

#[test]
fn test_config_multiple_keys() {
    let db = make_db();
    db.set_config("a", "1").expect("a");
    db.set_config("b", "2").expect("b");
    db.set_config("c", "3").expect("c");
    assert_eq!(db.get_config("a").unwrap(), Some("1".into()));
    assert_eq!(db.get_config("b").unwrap(), Some("2".into()));
    assert_eq!(db.get_config("c").unwrap(), Some("3".into()));
}

#[test]
fn test_import_localized_strings_from_manifest() {
    let db = make_db();
    let temp_root = std::env::temp_dir().join(format!("d2r_resource_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_root).expect("create temp dir");
    let json_path = temp_root.join("item-names.json");
    std::fs::write(
        &json_path,
        r#"[
  {"Key":"Shako","enUS":"Shako","zhCN":"军帽","zhTW":"軍帽"},
  {"Key":"Raven Frost","enUS":"Raven Frost","zhCN":"乌鸦之霜","zhTW":"烏鴉之霜"}
]"#,
    ).expect("write json");

    let manifest = ResourceManifest {
        profile_id: "mod:testmod:3.2".into(),
        source_kind: "mod".into(),
        game_version: "3.2".into(),
        mod_name: "testmod".into(),
        game_root: "D:/Games/D2R".into(),
        excel_path: temp_root.to_string_lossy().to_string(),
        strings_path: temp_root.to_string_lossy().to_string(),
        strings_legacy_path: String::new(),
        active_language: "zhCN".into(),
        supported_languages: vec!["enUS".into(), "zhCN".into(), "zhTW".into()],
        txt_files: Vec::new(),
        json_files: vec![ResourceFileInfo {
            role: "item_names".into(),
            file_type: "json".into(),
            relation: "基础物品名本地化".into(),
            path: json_path.to_string_lossy().to_string(),
            exists: true,
            languages: vec!["enUS".into(), "zhCN".into(), "zhTW".into()],
        }],
        checksum: String::new(),
        vanilla_profile_id: None,
        source_path: String::new(),
        fallback_chain: vec!["zhCN -> enUS".into()],
        notes: Vec::new(),
    };

    let inserted = db.import_localized_strings_from_manifest(&manifest).expect("import localized");
    assert!(inserted >= 6, "2 个 key * 3 种语言，至少应插入 6 行");
    let count = db.count_localized_strings_for_profile("mod:testmod:3.2").expect("count localized");
    assert!(count >= 6);

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_upsert_resource_manifest_persists_game_root() {
    let db = make_db();
    let manifest = ResourceManifest {
        profile_id: "vanilla:2.7.3".into(),
        source_kind: "vanilla".into(),
        game_version: "2.7.3".into(),
        mod_name: "(原版)".into(),
        game_root: "D:/Games/D2R-273".into(),
        excel_path: "D:/Games/D2R-273/data/global/excel".into(),
        strings_path: String::new(),
        strings_legacy_path: String::new(),
        active_language: "zhCN".into(),
        vanilla_profile_id: None,
        supported_languages: vec!["enUS".into(), "zhCN".into()],
        txt_files: Vec::new(),
        json_files: Vec::new(),
        checksum: String::new(),
        source_path: String::new(),
        fallback_chain: vec!["zhCN -> enUS".into()],
        notes: Vec::new(),
    };

    let profile_id = db.upsert_resource_manifest(&manifest).expect("upsert manifest");
    let conn = db.get_connection();
    let stored: (String, String) = conn
        .query_row(
            "SELECT game_root, game_version FROM resource_profile WHERE id = ?1",
            [profile_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query resource profile");
    assert_eq!(stored.0, "D:/Games/D2R-273");
    assert_eq!(stored.1, "2.7.3");
}

// ═══════ Warehouse CRUD ═══════

fn make_warehouse_item(id: &str, code: &str) -> WarehousedItem {
    WarehousedItem {
        id: id.into(),
        item_code: code.into(),
        item_name: format!("Test {code}"),
        item_kind: "rune".into(),
        quality: Some("Normal".into()),
        simple_item: true,
        quantity: 1,
        profile_key: "vanilla:3.0".into(),
        game_version: "3.0".into(),
        mod_name: String::new(),
        raw_item_bits: vec![0x10, 0x00, 0x80, 0x00],
        raw_bit_length: 32,
        item_json: r#"{"test":true}"#.into(),
        stash_name: Some("test.d2i".into()),
        imported_at: "2026-07-05T10:00:00Z".into(),
        page_name: "Page[5]".into(),
        tags: String::new(),
        notes: String::new(),
        source_character: None,
        source_save_path: None,
        slot_equipped: None,
        page_index: 0,
        position_x: 0,
        position_y: 0,
        inv_width: 1,
        inv_height: 1,
    }
}

#[test]
fn test_warehouse_add_and_list() {
    let db = make_db();
    db.warehouse_add(&make_warehouse_item("w-001", "r01")).expect("add");
    db.warehouse_add(&make_warehouse_item("w-002", "r02")).expect("add");

    let all = db.warehouse_list_all().expect("list");
    assert_eq!(all.len(), 2);
}

#[test]
fn test_warehouse_list_by_context_filters_mod_and_version() {
    let db = make_db();
    let mut vanilla = make_warehouse_item("w-001", "r01");
    vanilla.profile_key = "vanilla:2.7".into();
    vanilla.game_version = "2.7".into();
    vanilla.mod_name = String::new();
    let mut modded = make_warehouse_item("w-002", "r02");
    modded.profile_key = "mod:testmod:3.2".into();
    modded.game_version = "3.2".into();
    modded.mod_name = "testmod".into();
    db.warehouse_add(&vanilla).expect("add vanilla");
    db.warehouse_add(&modded).expect("add modded");

    let vanilla_items = db.warehouse_list_by_context("", "2.7").expect("list vanilla context");
    assert_eq!(vanilla_items.len(), 1);
    assert_eq!(vanilla_items[0].id, "w-001");

    let modded_items = db.warehouse_list_by_context("testmod", "3.2").expect("list mod context");
    assert_eq!(modded_items.len(), 1);
    assert_eq!(modded_items[0].id, "w-002");
}

#[test]
fn test_warehouse_list_by_profile_filters_exact_profile_key() {
    let db = make_db();
    let mut a = make_warehouse_item("w-001", "r01");
    a.profile_key = "vanilla:2.7".into();
    a.game_version = "2.7".into();
    let mut b = make_warehouse_item("w-002", "r02");
    b.profile_key = "mod:testmod:2.7".into();
    b.mod_name = "testmod".into();
    b.game_version = "2.7".into();
    db.warehouse_add(&a).expect("add profile a");
    db.warehouse_add(&b).expect("add profile b");

    let vanilla = db.warehouse_list_by_profile("vanilla:2.7").expect("list vanilla profile");
    assert_eq!(vanilla.len(), 1);
    assert_eq!(vanilla[0].id, "w-001");

    let modded = db.warehouse_list_by_profile("mod:testmod:2.7").expect("list mod profile");
    assert_eq!(modded.len(), 1);
    assert_eq!(modded[0].id, "w-002");
}

#[test]
fn test_warehouse_get_and_remove_respect_profile_key() {
    let db = make_db();
    let mut item = make_warehouse_item("w-001", "r01");
    item.profile_key = "vanilla:2.7".into();
    db.warehouse_add(&item).expect("add warehouse item");

    assert!(db
        .warehouse_get_in_profile("mod:testmod:2.7", "w-001")
        .expect("get in wrong profile")
        .is_none());
    assert!(!db
        .warehouse_remove_in_profile("mod:testmod:2.7", "w-001")
        .expect("remove in wrong profile"));

    assert!(db
        .warehouse_get_in_profile("vanilla:2.7", "w-001")
        .expect("get in correct profile")
        .is_some());
    assert!(db
        .warehouse_remove_in_profile("vanilla:2.7", "w-001")
        .expect("remove in correct profile"));
}

#[test]
fn test_warehouse_pages_in_profile_are_isolated() {
    let db = make_db();
    let mut a = make_warehouse_item("w-001", "r01");
    a.profile_key = "vanilla:2.7".into();
    a.page_name = "符文收藏".into();
    let mut b = make_warehouse_item("w-002", "r02");
    b.profile_key = "mod:testmod:2.7".into();
    b.mod_name = "testmod".into();
    b.page_name = "模组收藏".into();
    db.warehouse_add(&a).expect("add page a");
    db.warehouse_add(&b).expect("add page b");

    let vanilla_pages = db
        .warehouse_list_pages_in_profile("vanilla:2.7")
        .expect("list pages in vanilla profile");
    assert_eq!(vanilla_pages, vec!["符文收藏".to_string()]);

    let modded_items = db
        .warehouse_list_by_page_in_profile("mod:testmod:2.7", "模组收藏")
        .expect("list page in mod profile");
    assert_eq!(modded_items.len(), 1);
    assert_eq!(modded_items[0].id, "w-002");
}

#[test]
fn test_warehouse_update_meta_respects_profile_key() {
    let db = make_db();
    let mut item = make_warehouse_item("w-001", "r01");
    item.profile_key = "vanilla:2.7".into();
    db.warehouse_add(&item).expect("add warehouse item");

    assert!(!db
        .warehouse_update_meta_in_profile("mod:testmod:2.7", "w-001", "模组页", "x", "y")
        .expect("update wrong profile"));
    assert!(db
        .warehouse_update_meta_in_profile("vanilla:2.7", "w-001", "符文页", "rune", "ok")
        .expect("update correct profile"));

    let updated = db
        .warehouse_get_in_profile("vanilla:2.7", "w-001")
        .expect("get updated item")
        .expect("item should exist");
    assert_eq!(updated.page_name, "符文页");
    assert_eq!(updated.tags, "rune");
    assert_eq!(updated.notes, "ok");
}

#[test]
fn test_warehouse_get_by_id() {
    let db = make_db();
    db.warehouse_add(&make_warehouse_item("w-001", "r01")).ok();
    let got = db.warehouse_get("w-001").expect("get");
    assert!(got.is_some());
    let got = got.unwrap();
    assert_eq!(got.item_code, "r01");
}

#[test]
fn test_warehouse_remove() {
    let db = make_db();
    db.warehouse_add(&make_warehouse_item("w-001", "r01")).ok();
    let removed = db.warehouse_remove("w-001").expect("remove");
    assert!(removed);
    let got = db.warehouse_get("w-001").expect("get after");
    assert!(got.is_none());
}

#[test]
fn test_warehouse_remove_nonexistent() {
    let db = make_db();
    let removed = db.warehouse_remove("w-missing").expect("remove");
    assert!(!removed);
}

#[test]
fn test_warehouse_list_pages() {
    let db = make_db();
    db.warehouse_add(&make_warehouse_item("w-001", "r01")).ok();
    db.warehouse_add(&make_warehouse_item("w-002", "r02")).ok();

    // Same page name for both
    let pages = db.warehouse_list_pages().expect("pages");
    assert!(!pages.is_empty(), "should have at least 1 page");
    assert!(pages.iter().any(|p| p == "Page[5]"), "Page[5] should appear");
}

#[test]
fn test_warehouse_list_by_page() {
    let db = make_db();
    let mut item_a = make_warehouse_item("w-001", "r01");
    item_a.page_name = "Gems".into();
    let mut item_b = make_warehouse_item("w-002", "gcv");
    item_b.page_name = "Runes".into();

    db.warehouse_add(&item_a).ok();
    db.warehouse_add(&item_b).ok();

    let gems = db.warehouse_list_by_page("Gems").expect("by page");
    let runes = db.warehouse_list_by_page("Runes").expect("by page 2");
    assert_eq!(gems.len(), 1);
    assert_eq!(runes.len(), 1);
}

#[test]
fn test_warehouse_update_meta() {
    let db = make_db();
    db.warehouse_add(&make_warehouse_item("w-001", "r01")).ok();
    let updated = db.warehouse_update_meta("w-001", "UpdatedPage", "tag1,tag2", "some notes")
        .expect("update");
    assert!(updated, "should return true if row exists");

    let got = db.warehouse_get("w-001").expect("get").unwrap();
    assert_eq!(got.page_name, "UpdatedPage");
    assert_eq!(got.tags, "tag1,tag2");
    assert_eq!(got.notes, "some notes");
}

// ═══════ Integration workflow ═══════

#[test]
fn test_full_marketplace_workflow() {
    let db = make_db();
    let starting_bal = db.get_token_balance().unwrap();
    assert_eq!(starting_bal, 10000);

    // 1. User lists a rune
    let item = make_item("v-el-001", "r01", 1);
    db.add_virtual_item(&item).expect("list");
    assert_eq!(db.get_listed_items().unwrap().len(), 1);

    // 2. Update balance (simulate marketplace cut)
    db.update_token_balance(-6900).expect("cut");
    assert_eq!(db.get_token_balance().unwrap(), 3100);

    // 3. Cancel the listing
    let cancelled = db.mark_listing_cancelled("v-el-001", "vanilla:default").expect("cancel");
    assert!(cancelled);

    // 4. Verify no longer listed
    assert_eq!(db.get_listed_items().unwrap().len(), 0);

    // 5. Add warehouse item
    let wi = make_warehouse_item("w-el-001", "r01");
    db.warehouse_add(&wi).expect("warehouse");
    assert_eq!(db.warehouse_list_all().unwrap().len(), 1);

    // 6. Save config
    db.set_config("last_listing", "v-el-001").expect("config");
    assert_eq!(db.get_config("last_listing").unwrap(), Some("v-el-001".into()));
}
