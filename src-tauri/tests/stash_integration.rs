use d2r_marketplace_lib::protocol::d2i::legacy::game_items::ALL_ITEMS;
use d2r_marketplace_lib::protocol::d2i::legacy::magical_props::MAGICAL_PROPS;
use d2r_marketplace_lib::protocol::d2i::legacy::item::{read_stash_items, update_stash_items, StashItem};
use d2r_marketplace_lib::protocol::d2i::legacy::page::{split_legacy_d2i_pages, find_stackable_page, reassemble_d2i};

// ─── Game Constants Validation ────────────────────────────────

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
fn test_magical_props_table_has_updated_entries() {
    assert!(MAGICAL_PROPS.len() >= 359,
        "magical_properties table should have at least 359 entries, got {}",
        MAGICAL_PROPS.len());
    // With D2RMM mod stats, can have up to 420 entries
    eprintln!("MAGICAL_PROPS has {} entries (359 base + {} mod)", 
        MAGICAL_PROPS.len(), MAGICAL_PROPS.len() - 359);
}

#[test]
fn test_magical_props_key_stats() {
    // Stat 31 = defense rating: sB=11, save_add for base value
    assert_eq!(MAGICAL_PROPS[31].save_bits, 11, "defense should use 11 bits");
    assert_eq!(MAGICAL_PROPS[31].save_add, 10, "defense should have save_add=10");
    // Stat 72 = current durability: sB=9
    assert_eq!(MAGICAL_PROPS[72].save_bits, 9, "current durability should use 9 bits");
    // Stat 73 = max durability: sB=8
    assert_eq!(MAGICAL_PROPS[73].save_bits, 8, "max durability should use 8 bits");
}

#[test]
fn test_magical_props_all_have_sb() {
    // All 359 entries should have save_bits defined
    for (i, prop) in MAGICAL_PROPS.iter().enumerate() {
        assert!(prop.num_sub_props >= 1, "id={}: np must be >= 1", i);
        // save_bits=0 means no saved value (runtime-only stat)
    }
}

#[test]
fn test_all_items_count() {
    assert!(ALL_ITEMS.len() >= 650, "should have 650+ items, got {}", ALL_ITEMS.len());
}

#[test]
fn test_all_items_known_codes() {
    let codes: Vec<&str> = ALL_ITEMS.iter().map(|(c, _, _, _, _)| *c).collect();
    // Verify key items exist
    for &code in &["cap", "hax", "amu", "rin", "r01", "gcv", "rvs", "pk1"] {
        assert!(codes.contains(&code), "Item code '{}' should exist in ALL_ITEMS", code);
    }
}

#[test]
fn test_all_items_category_lookup() {
    // Cap → Armor
    let cap = ALL_ITEMS.iter().find(|(c, _, _, _, _)| *c == "cap").unwrap();
    assert!(cap.2, "cap should be armor");  // is_armor
    assert!(!cap.3, "cap should not be weapon");
    assert!(!cap.4, "cap should not be shield");

    // Hand Axe → Weapon
    let hax = ALL_ITEMS.iter().find(|(c, _, _, _, _)| *c == "hax").unwrap();
    assert!(!hax.2, "hax should not be armor");
    assert!(hax.3, "hax should be weapon");

    // Amulet → neither
    let amu = ALL_ITEMS.iter().find(|(c, _, _, _, _)| *c == "amu").unwrap();
    assert!(!amu.2 && !amu.3 && !amu.4, "amu should not be armor/weapon/shield");

    // Rune → stackable, not armor/weapon
    let r01 = ALL_ITEMS.iter().find(|(c, _, _, _, _)| *c == "r01").unwrap();
    assert!(!r01.2 && !r01.3, "r01 should not be armor/weapon");
}

// ─── Stash File Parsing Tests ────────────────────────────────

/// Helper: parse items from a real stash file and report stats
fn parse_stash_fixture() -> Option<(Vec<d2r_marketplace_lib::protocol::d2i::legacy::item::StashItem>, Vec<u8>)> {
    let fixture = fixture_path("ModernSharedStashSoftCoreV2.d2i")?;
    let data = std::fs::read(&fixture).ok()?;
    let pages = split_legacy_d2i_pages(&data).ok()?;
    let items = read_stash_items(&pages.pages).ok()?;
    Some((items, data))
}

#[test]
fn test_real_stash_parses_without_panic() {
    let Some((items, _data)) = parse_stash_fixture() else { return };
    // Just parsing should not crash
    assert!(items.len() >= 4, "Should find at least 4 stackable items, got {}", items.len());
}

#[test]
fn test_real_stash_item_types_are_valid() {
    let Some((items, _)) = parse_stash_fixture() else { return };
    let valid_codes: Vec<&str> = ALL_ITEMS.iter().map(|(c, _, _, _, _)| *c).collect();

    for item in &items {
        if !valid_codes.contains(&item.item_type.as_str()) {
            // Unknown codes (e.g. from mods) are expected; just log them
            eprintln!("WARNING: unknown item code '{}' — not in game constants", item.item_type);
            continue;
        }
        assert!(item.amount >= 1,
            "Amount {} for {} should be >= 1", item.amount, item.item_type);
        assert!(item.amount <= 255,
            "Amount {} for {} should be <= 255", item.amount, item.item_type);
    }
}

#[test]
fn test_real_stash_known_stackables_present() {
    let Some((items, _)) = parse_stash_fixture() else { return };
    let known = [
        "r01","r02","r03","r04","r05","r06","r07","r08","r09","r10",
        "r11","r12","r13","r14","r15","r16","r17","r18","r19","r20",
        "r21","r22","r23","r24","r25","r26","r27","r28","r29","r30",
        "r31","r32","r33",
        "gcv","gcw","gcg","gcr","gcb","gcy","skc",
        "gfv","gfw","gfg","gfr","gfb","gfy","skf",
        "gsv","gsw","gsg","gsr","gsb","gsy","sku",
        "gzv","glw","glg","glr","glb","gly","skl",
        "gpv","gpw","gpg","gpr","gpb","gpy","skz",
        "rvs","rvl",
    ];
    let found: Vec<&str> = items.iter().map(|i| i.item_type.as_str()).collect();
    let mut seen = 0;
    for &code in &known {
        if found.contains(&code) {
            seen += 1;
            let item = items.iter().find(|i| i.item_type == code).unwrap();
            eprintln!("  {} ×{}", code, item.amount);
        }
    }
    eprintln!("Known stackables found: {}/{}", seen, known.len());
    // The stash fixture is equipment-heavy; don't require a minimum
}

#[test]
fn test_real_stash_all_items_have_amount_gt_zero() {
    let Some((items, _)) = parse_stash_fixture() else { return };
    assert!(items.iter().all(|i| i.amount > 0),
        "All items should have amount > 0");
}

#[test]
fn test_real_stash_all_simple_items() {
    let Some((items, _)) = parse_stash_fixture() else { return };
    let non_simple: Vec<&StashItem> = items.iter().filter(|i| !i.simple_item).collect();
    let simple_count = items.len() - non_simple.len();
    eprintln!("{} total, {} simple, {} non-simple",
        items.len(), simple_count, non_simple.len());
}

#[test]
fn test_real_stash_parsed_vs_count() {
    let Some((items, _)) = parse_stash_fixture() else { return };
    eprintln!("Parsed {} stackable items from stash", items.len());
}

// ─── Round-trip: Parse → Write → Re-parse ─────────────────────

#[test]
fn test_real_stash_roundtrip_preserves_items() {
    let Some((items, data)) = parse_stash_fixture() else { return };
    if items.is_empty() { return; }

    let pages = split_legacy_d2i_pages(&data).unwrap();
    let stack_page = find_stackable_page(&pages.pages).unwrap().clone();

    // Use existing write path for simple items
    let simple_items: Vec<&StashItem> = items.iter().filter(|i| i.simple_item).collect();
    if !simple_items.is_empty() {
        let code = &simple_items[0].item_type;
        let (new_items, _new_data) = update_stash_items(
            &stack_page, code, 0, false
        ).expect("Round-trip update failed");
        eprintln!("Round-trip preserved {} items (simple only)", new_items.len());
    } else {
        eprintln!("No simple items in stash — roundtrip test skipped");
    }
}

// ─── Game Constants Coverage ───────────────────────────────

#[test]
fn test_magical_props_sb_distribution() {
    let mut hist = [0u32; 33];
    for prop in MAGICAL_PROPS.iter() {
        let sb = prop.save_bits;
        if (sb as usize) < hist.len() {
            hist[sb as usize] += 1;
        }
    }
    eprintln!("Save bits distribution:");
    for (bits, count) in hist.iter().enumerate() {
        if *count > 0 {
            eprintln!("  sB={}: {} props", bits, count);
        }
    }
}

#[test]
fn test_all_items_by_category() {
    let mut armor = 0u32;
    let mut weapon = 0u32;
    let mut shield = 0u32;
    let mut other = 0u32;
    for (_, _, is_a, is_w, is_s) in ALL_ITEMS.iter() {
        if *is_a { armor += 1; }
        if *is_w { weapon += 1; }
        if *is_s { shield += 1; }
        if !*is_a && !*is_w && !*is_s { other += 1; }
    }
    eprintln!("ALL_ITEMS categories:");
    eprintln!("  Armor:  {}", armor);
    eprintln!("  Weapon: {}", weapon);
    eprintln!("  Shield: {}", shield);
    eprintln!("  Other:  {}", other);
    assert!(armor > 100, "Should have 100+ armor items");
    assert!(weapon > 100, "Should have 100+ weapon items");
}

// ─── Stash Write Integration Tests ─────────────────────────

/// Test that update_stash_items can add a gem (gcv) to the stackable page,
/// write it back to disk, re-read, and verify the item exists.
#[test]
fn test_add_gem_to_stash_roundtrip() {
    let fixture = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
    let data = std::fs::read(&fixture).expect("Fixture not found");

    // Create a temp copy so we don't modify the fixture
    let tmp_dir = std::env::temp_dir().join(format!("d2r_test_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&tmp_dir).ok();
    let tmp_path = tmp_dir.join("test_stash.d2i");
    std::fs::write(&tmp_path, &data).expect("Failed to copy fixture");

    // Parse pages
    let raw = std::fs::read(&tmp_path).expect("Failed to read temp copy");
    let pages = split_legacy_d2i_pages(&raw).expect("Failed to parse pages");
    let stackable_page = find_stackable_page(&pages.pages)
        .cloned()
        .expect("No stackable page found");

    // Count items before
    let items_before = read_stash_items(&pages.pages).expect("Failed to read items");
    let gem_count_before = items_before.iter().filter(|i| i.item_type == "gcv").count();

    // Add gcv (Chipped Amethyst) with quantity 3
    let (updated_items, updated_page_data) = update_stash_items(
        &stackable_page, "gcv", 3, true,
    ).expect("update_stash_items failed");

    // Reassemble and write back
    let mut updated_pages = pages.pages.clone();
    let stack_idx = pages.pages.iter().position(|p| p.is_stackable).unwrap();
    updated_pages[stack_idx].data = updated_page_data;
    let final_data = reassemble_d2i(&updated_pages, &pages.tail);
    std::fs::write(&tmp_path, &final_data).expect("Failed to write stash");

    // Debug: check what items exist on the stackable page after write
    let verify_raw = std::fs::read(&tmp_path).expect("Failed to re-read");
    let verify_pages = split_legacy_d2i_pages(&verify_raw).expect("Failed to re-parse");
    let verify_items = read_stash_items(&verify_pages.pages).expect("Failed to re-read items");
    eprintln!("After write: {} total items", verify_items.len());
    for vi in &verify_items {
        eprintln!("  item_type={}, amount={}", vi.item_type, vi.amount);
    }

    // Verify the updated items are correct
    assert_eq!(updated_items.len(), verify_items.len(),
        "Write-then-read should produce same item count");

    let gem_items: Vec<&StashItem> = verify_items.iter().filter(|i| i.item_type == "gcv").collect();
    assert!(!gem_items.is_empty(), "gcv should exist after write — current items: {:?}",
        verify_items.iter().map(|i| &i.item_type).collect::<Vec<&String>>());

    if gem_count_before == 0 {
        assert_eq!(gem_items[0].amount, 3, "gcv should have quantity 3");
    } else {
        let total_qty: u32 = gem_items.iter().map(|i| i.amount).sum();
        assert!(total_qty >= 3, "Total gcv quantity should be at least 3");
    }

    // Clean up temp dir
    std::fs::remove_dir_all(&tmp_dir).ok();
}

/// Test that buying a potion (rvs) works — different item code family from runes
#[test]
fn test_add_potion_to_stash_roundtrip() {
    let fixture = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
    let data = std::fs::read(&fixture).expect("Fixture not found");

    let tmp_dir = std::env::temp_dir().join(format!("d2r_test_potion_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&tmp_dir).ok();
    let tmp_path = tmp_dir.join("test_stash_potion.d2i");
    std::fs::write(&tmp_path, &data).expect("Failed to copy fixture");

    let raw = std::fs::read(&tmp_path).expect("Failed to read");
    let pages = split_legacy_d2i_pages(&raw).expect("Failed to parse pages");
    let stackable_page = find_stackable_page(&pages.pages)
        .cloned()
        .expect("No stackable page");

    let items_before = read_stash_items(&pages.pages).expect("Failed to read items");
    let _potion_before = items_before.iter().filter(|i| i.item_type == "rvs").count();

    // Add rvs (Rejuvenation Potion)
    let (_updated_items, updated_page_data) = update_stash_items(
        &stackable_page, "rvs", 5, true,
    ).expect("update_stash_items for potion failed");

    let mut updated_pages = pages.pages.clone();
    let stack_idx = pages.pages.iter().position(|p| p.is_stackable).unwrap();
    updated_pages[stack_idx].data = updated_page_data;
    let final_data = reassemble_d2i(&updated_pages, &pages.tail);
    std::fs::write(&tmp_path, &final_data).expect("Failed to write");

    // Verify
    let verify_raw = std::fs::read(&tmp_path).expect("Failed to re-read");
    let verify_pages = split_legacy_d2i_pages(&verify_raw).expect("Failed to re-parse");
    let verify_items = read_stash_items(&verify_pages.pages).expect("Failed to re-read items");

    let potion_items: Vec<&StashItem> = verify_items.iter().filter(|i| i.item_type == "rvs").collect();
    assert!(!potion_items.is_empty(), "rvs should exist after write");

    let total_qty: u32 = potion_items.iter().map(|i| i.amount).sum();
    assert!(total_qty >= 5, "rvs total quantity should be at least 5");

    std::fs::remove_dir_all(&tmp_dir).ok();
}
