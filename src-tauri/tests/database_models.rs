//! Database models 综合测试
//!
//! 验证:
//! - Serde JSON round-trip (Serialize → Deserialize 字段保真)
//! - ListedItem / SoldItem / WarehousedItem / Transaction 字段
//! - Frontend 兼容的 JSON 序列化字段名
//! - Optional 字段 None 序列化正确

use d2r_marketplace_lib::database::models::{
    AppConfig, ListedItem, SoldItem, Transaction, VirtualItem, WarehousedItem,
};

#[test]
fn test_virtual_item_serde_round_trip() {
    let item = VirtualItem {
        id: "v-001".into(),
        name: "El Rune".into(),
        item_code: Some("r01".into()),
        item_kind: Some("rune".into()),
        item_type: Some("El".into()),
        quality: Some("Normal".into()),
        level: Some(11),
        attributes: Some("Weapon Damage +3".into()),
        source: Some("stash".into()),
        exported_from: None,
        purchased_at: None,
        token_price: Some(6900),
        status: Some("listed".into()),
        quantity: Some(1),
        unit_price: Some(6900),
        listed_at: Some("2026-07-05".into()),
        sell_after_seconds: Some(3600),
        profile_id: Some(42),
        profile_key: Some("mod:testmod:3.2".into()),
        game_version: Some("3.2".into()),
        mod_name: Some("testmod".into()),
    };
    let json = serde_json::to_string(&item).expect("serialize");
    let back: VirtualItem = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.id, "v-001");
    assert_eq!(back.item_code, Some("r01".into()));
    assert_eq!(back.token_price, Some(6900));
    assert_eq!(back.exported_from, None);
    assert_eq!(back.profile_key, Some("mod:testmod:3.2".into()));
}

#[test]
fn test_listed_item_serde() {
    let li = ListedItem {
        id: "L001".into(),
        name: "El".into(),
        quantity: 1,
        unit_price: 6900,
        listed_at: Some("2026-07-05T10:00:00Z".into()),
        sell_after_seconds: 3600,
        status: Some("active".into()),
        item_code: None,
        item_kind: None,
        quality: None,
        listed_by: None,
    };
    let json = serde_json::to_string(&li).expect("ser");
    let back: ListedItem = serde_json::from_str(&json).expect("des");
    assert_eq!(back.unit_price, 6900);
    assert_eq!(back.sell_after_seconds, 3600);
}

#[test]
fn test_sold_item_required_fields() {
    // SoldItem.date is String (not Option) - required field
    let si = SoldItem {
        id: "S001".into(),
        name: "El Rune".into(),
        quantity: 1,
        unit_price: 6900,
        listed_at: "2026-07-05T10:00:00Z".into(),
        sell_after_seconds: 3600,
    };
    let json = serde_json::to_string(&si).expect("ser");
    let back: SoldItem = serde_json::from_str(&json).expect("des");
    assert_eq!(back.listed_at, "2026-07-05T10:00:00Z");
}

#[test]
fn test_transaction_round_trip() {
    let tx = Transaction {
        id: Some(42),
        tx_type: "sell".into(),
        item_id: Some("v-001".into()),
        token_amount: 6900,
        description: "Sold El Rune".into(),
        date: Some("2026-07-05T10:30:00Z".into()),
    };
    let json = serde_json::to_string(&tx).expect("ser");
    let back: Transaction = serde_json::from_str(&json).expect("des");
    assert_eq!(back.id, Some(42));
    assert_eq!(back.token_amount, 6900);
}

#[test]
fn test_app_config_round_trip() {
    let cfg = AppConfig {
        key: "save_folder".into(),
        value: "D:/d2_saves/".into(),
    };
    let json = serde_json::to_string(&cfg).expect("ser");
    let back: AppConfig = serde_json::from_str(&json).expect("des");
    assert_eq!(back.key, "save_folder");
}

#[test]
fn test_warehoused_item_required_fields() {
    let wi = WarehousedItem {
        id: "w-001".into(),
        item_code: "r01".into(),
        item_name: "El Rune".into(),
        item_kind: "rune".into(),
        quality: Some("Normal".into()),
        simple_item: true,
        quantity: 1,
        profile_key: "vanilla:3.0".into(),
        game_version: "3.0".into(),
        mod_name: "".into(),
        raw_item_bits: vec![0x10, 0x00, 0x80, 0x00, 0x05, 0x68, 0x74, 0x67],
        raw_bit_length: 64,
        item_json: r#"{"name":"El"}"#.into(),
        stash_name: Some("ModernSharedStashSoftCoreV2.d2i".into()),
        imported_at: "2026-07-05T10:00:00Z".into(),
        page_name: "SharedStashSoftCore".into(),
        tags: "rune,low-tier".into(),
        notes: String::new(),
        source_character: Some("EchoingStrike".into()),
        source_save_path: Some("C:\\save\\EchoingStrike.d2s".into()),
        slot_equipped: None,
        page_index: 0,
        position_x: 0,
        position_y: 0,
        inv_width: 1,
        inv_height: 1,
    };
    let json = serde_json::to_string(&wi).expect("ser");
    let back: WarehousedItem = serde_json::from_str(&json).expect("des");
    assert_eq!(back.item_code, "r01");
    assert_eq!(back.raw_item_bits.len(), 8);
    assert_eq!(back.raw_bit_length, 64);
    assert_eq!(back.source_character, Some("EchoingStrike".into()));
    assert_eq!(back.slot_equipped, None);
}
#[test]
fn test_warehoused_default_mod_name() {
    let wi = WarehousedItem {
        id: "w-002".into(),
        item_code: "gcv".into(),
        item_name: "Chipped Amethyst".into(),
        item_kind: "gem".into(),
        quality: None,
        simple_item: true,
        quantity: 3,
        profile_key: "vanilla:3.0".into(),
        game_version: "3.0".into(),
        mod_name: String::new(),
        raw_item_bits: vec![],
        raw_bit_length: 0,
        item_json: String::new(),
        stash_name: None,
        imported_at: "2026-07-05".into(),
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
    };
    let json = serde_json::to_string(&wi).expect("ser");
    assert!(json.contains("\"mod_name\":\"\""), "default empty mod_name should serialize as empty string");
    assert!(json.contains("\"source_character\":null"), "None source should serialize as null");
}

#[test]
fn test_json_frontend_field_compatibility() {
    // Frontend uses snake_case field names via serde rename_all
    // Verify the actual JSON output matches what the frontend expects
    let li = ListedItem {
        id: "L001".into(),
        name: "El".into(),
        quantity: 1,
        unit_price: 6900,
        listed_at: None,
        sell_after_seconds: 3600,
        status: None,
        item_code: None,
        item_kind: None,
        quality: None,
        listed_by: None,
    };
    let v = serde_json::to_value(&li).expect("to_value");
    assert!(v.get("id").is_some());
    assert!(v.get("quantity").is_some());
    assert!(v.get("unit_price").is_some());
    assert!(v.get("sell_after_seconds").is_some());
    assert_eq!(v["unit_price"], 6900);
    assert_eq!(v["sell_after_seconds"], 3600);
}

#[test]
fn test_virtual_item_some_default() {
    let vi = VirtualItem {
        id: String::new(),
        name: String::new(),
        item_code: None,
        item_kind: None,
        item_type: None,
        quality: None,
        level: None,
        attributes: None,
        source: None,
        exported_from: None,
        purchased_at: None,
        token_price: None,
        status: None,
        quantity: None,
        unit_price: None,
        listed_at: None,
        sell_after_seconds: None,
        profile_id: None,
        profile_key: None,
        game_version: None,
        mod_name: None,
    };
    let json = serde_json::to_string(&vi).expect("empty serialize");
    let back: VirtualItem = serde_json::from_str(&json).expect("empty deserialize");
    assert_eq!(back.id, "");
    assert_eq!(back.item_code, None);
}
