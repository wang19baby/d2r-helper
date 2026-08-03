//! Character equipment extraction tests.
//! Tests conversion of parsed d2s items → WarehousedItem entries.

use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures").join(name)
}

use d2r_marketplace_lib::protocol::common::{ItemLocation, ItemMode, ItemQuality};
use d2r_marketplace_lib::database::models::WarehousedItem;

fn location_to_slot(loc: &ItemLocation) -> Option<&'static str> {
    match loc {
        ItemLocation::Head => Some("helm"),
        ItemLocation::Neck => Some("amulet"),
        ItemLocation::Torso => Some("armor"),
        ItemLocation::RightHand => Some("weapon_main"),
        ItemLocation::LeftHand => Some("shield_main"),
        ItemLocation::RightFinger => Some("ring_l"),
        ItemLocation::LeftFinger => Some("ring_r"),
        ItemLocation::Waist => Some("belt"),
        ItemLocation::Feet => Some("boots"),
        ItemLocation::Hands => Some("gloves"),
        _ => None,
    }
}

fn quality_str(q: &ItemQuality) -> Option<&'static str> {
    match q {
        ItemQuality::Unique => Some("unique"),
        ItemQuality::Set => Some("set"),
        ItemQuality::Rare => Some("rare"),
        ItemQuality::Magic => Some("magic"),
        ItemQuality::Superior => Some("superior"),
        ItemQuality::Low => Some("low"),
        ItemQuality::Normal => Some("normal"),
        _ => None,
    }
}

/// Extract all items from a parsed d2s into WarehousedItems.
fn extract_all(
    file: &d2r_marketplace_lib::protocol::d2s::parser::D2SCharacter,
    save_path: &str,
) -> Vec<WarehousedItem> {
    let source_char = &file.header.name;
    let mut items: Vec<WarehousedItem> = Vec::new();

    let all_pi: Vec<&d2r_marketplace_lib::protocol::d2i::parser::ParsedItem> = file
        .equipped
        .iter()
        .chain(file.belt.iter())
        .chain(file.backpack.iter())
        .chain(file.cube.iter())
        .chain(file.merc.iter())
        .collect();
    for pi in &all_pi {
        let code = &pi.item.code;
        let kind = d2r_marketplace_lib::protocol::d2i::legacy::constants::ITEM_CODE_MAP.iter()
            .find(|(c, _, _, _)| *c == code.as_str())
            .map(|(_, _, k, _)| k.to_string())
            .unwrap_or_else(|| "misc".to_string());

        // Determine slot name for equipped items
        let slot = if pi.item.mode == ItemMode::Equipped {
            location_to_slot(&pi.item.location)
        } else {
            None
        };

        items.push(WarehousedItem {
            id: format!("{}-{}-{}", source_char, code, items.len()),
            item_code: code.clone(),
            item_name: String::new(),
            item_kind: kind,
            quality: quality_str(&pi.item.quality).map(|s| s.to_string()),
            simple_item: pi.item.flags.simple_item(),
            quantity: pi.item.amount.max(1),
            profile_key: "test:profile".to_string(),
            game_version: String::new(),
            mod_name: String::new(),
            raw_item_bits: vec![],
            raw_bit_length: 0,
            item_json: String::new(),
            stash_name: Some(save_path.to_string()),
            imported_at: "2026-07-09T00:00:00Z".to_string(),
            page_name: "角色装备".to_string(),
            tags: String::new(),
            notes: String::new(),
            source_character: Some(source_char.clone()),
            source_save_path: Some(save_path.to_string()),
            slot_equipped: slot.map(|s| s.to_string()),
            page_index: 0,
            position_x: 0,
            position_y: 0,
            inv_width: 1,
            inv_height: 1,
        });
    }
    items
}

#[test]
fn test_extract_librarian_items() {
    let fp = fixture("librarian.d2s");
    if !fp.exists() {
        eprintln!("SKIP: fixture librarian.d2s 未随仓库分发");
        return;
    }
    let data = std::fs::read(&fp).expect("read fixture");
    let file = d2r_marketplace_lib::protocol::d2s::parser::parse_file(&data)
        .expect("parse d2s");

    let save_path = fixture("librarian.d2s").to_string_lossy().to_string();
    let items = extract_all(&file, &save_path);

    let equipped_count = items.iter().filter(|w| w.slot_equipped.is_some()).count();
    let backpack_count = items.iter().filter(|w| w.slot_equipped.is_none()).count();

    println!("=== Extract: {} ({}) ===", file.header.name, items.len());
    println!("  equipped={} backpack={}", equipped_count, backpack_count);
    for wi in &items {
        let loc = if wi.slot_equipped.is_some() { "E" } else { "B" };
        println!("  [{}] code={:4} q={:?} slot={:?}", loc, wi.item_code, wi.quality, wi.slot_equipped);
    }

    assert!(!items.is_empty(), "should extract at least 1 item");
    assert!(items.iter().all(|w| w.source_character == Some(file.header.name.clone())),
        "all items must have source_character");
}
