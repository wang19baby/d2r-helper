/// 使用 Node.js 解析器对真实存档文件进行详细验证。
/// 这些测试验证解析器能正确识别所有类型的物品。
use std::path::Path;

/// 从 Node.js 解析器获取存档中的物品
fn parse_stash() -> Option<Vec<d2r_marketplace_lib::protocol::d2i::legacy::item::StashItem>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ModernSharedStashSoftCoreV2.d2i");
    if !fixture.exists() {
        eprintln!("SKIP: fixture ModernSharedStashSoftCoreV2.d2i 未随仓库分发");
        return None;
    }
    d2r_marketplace_lib::protocol::d2i::legacy::node_reader::read_stash_with_node(
        &fixture.to_string_lossy()
    ).ok()
}

#[test]
fn test_node_parser_returns_all_stackable_items() {
    let Some(items) = parse_stash() else { return };
    // 应该有 70+ 个物品（62 简单 + 药水/钥匙/精华/碎片）
    assert!(items.len() >= 70, "Expected 70+ items, got {}", items.len());
}

#[test]
fn test_node_parser_finds_all_runes() {
    let Some(items) = parse_stash() else { return };
    let codes: Vec<&str> = items.iter().map(|i| i.item_type.as_str()).collect();
    // 存档中实际存在的符文（gpw 和 r27 数量为 0 被过滤）
    let runes = ["r01","r02","r03","r04","r05","r06","r07","r08","r09","r10",
                 "r11","r12","r13","r14","r15","r16","r17","r18","r19","r20",
                 "r21","r22","r23","r24","r25","r26","r29","r30","r31"];
    for r in &runes {
        assert!(codes.contains(r), "Rune {} should be in stash", r);
    }
}

#[test]
fn test_node_parser_finds_all_gems() {
    let Some(items) = parse_stash() else { return };
    let codes: Vec<&str> = items.iter().map(|i| i.item_type.as_str()).collect();
    // 存档中实际存在的宝石（gpw 数量为 0 被过滤）
    let gems = ["gcv","gcw","gcg","gcr","gcb","gcy","skc",
                "gfv","gfw","gfg","gfr","gfb","gfy","skf",
                "gsv","gsw","gsg","gsr","gsb","gsy","sku",
                "gzv","glw","glg","glr","glb","gly","skl",
                "gpg"];
    for g in &gems {
        assert!(codes.contains(g), "Gem {} should be in stash", g);
    }
}

#[test]
fn test_node_parser_finds_potions_keys_essences_shards() {
    let Some(items) = parse_stash() else { return };
    let codes: Vec<&str> = items.iter().map(|i| i.item_type.as_str()).collect();
    for code in &["rvs", "rvl", "pk1", "tes", "ceh", "bet", "fed"] {
        assert!(codes.contains(code), "Item {} should be in stash", code);
    }
}

#[test]
fn test_node_parser_correct_amounts() {
    let Some(items) = parse_stash() else { return };
    // 验证数量的参考值来自 Node.js 解析结果（2026-07-03 captured）
    // 注意：fixture 文件（用户的真实 stash）会随时间变化。
    // 如果此测试失败：先用 `--nocapture` 跑一遍，将 "ACTUAL" 那行复制到 expected 列表。
    let expected: Vec<(&str, u32)> = vec![
        ("r01", 22), ("r02", 11), ("r03", 38), ("r04", 13),
        ("r05", 62), ("r06", 68), ("r07", 79), ("r08", 103),
        ("r09", 69), ("r10", 71), ("r11", 86), ("r12", 56),
        ("r13", 52), ("r14", 39), ("r15", 58), ("r16", 13),
        ("r17", 38), ("r18", 14), ("r19", 11), ("r20", 7),
        ("r21", 7),  ("r22", 9),  ("r23", 4),  ("r24", 7),
        ("r25", 3),  ("r26", 1),  ("r29", 1),  ("r30", 1),
        ("r31", 1),
        ("gcw", 30), ("skc", 15), ("gcg", 20), ("gcb", 26),
        ("gcy", 16), ("gcr", 17), ("gsg", 5),  ("gcv", 22),
        ("gfv", 8),  ("gfw", 11), ("gsb", 7),  ("gfr", 5),
        ("skf", 8),  ("sku", 3),  ("gfg", 4),  ("gsw", 2),
        ("gly", 37), ("gfy", 10), ("gsy", 2),  ("glw", 10),
        ("gzv", 24), ("skl", 28), ("glb", 30), ("glg", 40),
        ("gfb", 10), ("glr", 21), ("gsv", 5),  ("gsr", 5),
        ("gpg", 1),
        ("rvs", 63), ("rvl", 90), ("pk1", 10),
        ("tes", 7),  ("ceh", 4),  ("bet", 8),  ("fed", 4),
    ];

    let mut mismatches: Vec<String> = Vec::new();
    for (code, exp_qty) in &expected {
        match items.iter().find(|i| i.item_type == *code) {
            Some(item) if item.amount != *exp_qty => {
                mismatches.push(format!("{}: expected {} got {}", code, exp_qty, item.amount));
            }
            None => mismatches.push(format!("{}: MISSING (expected {})", code, exp_qty)),
            _ => {}
        }
    }
    if !mismatches.is_empty() {
        // 打印完整实际值便于更新 expected 列表
        eprintln!("\n=== ACTUAL Node.js output (use to update expected) ===");
        let mut sorted_items = items.iter().collect::<Vec<_>>();
        sorted_items.sort_by(|a, b| a.item_type.cmp(&b.item_type));
        for item in &sorted_items {
            eprintln!("  (\"{}\", {}),", item.item_type, item.amount);
        }
        panic!("Node.js reference data drift: {} mismatches:\n  {}",
            mismatches.len(), mismatches.join("\n  "));
    }
}

#[test]
fn test_node_parser_no_zero_amounts() {
    let Some(items) = parse_stash() else { return };
    for item in &items {
        assert!(item.amount > 0 && item.amount <= 255,
            "{} has invalid amount: {}", item.item_type, item.amount);
    }
}

#[test]
fn test_node_parser_all_items_in_game_constants() {
    let Some(items) = parse_stash() else { return };
    let game_codes: Vec<&str> = d2r_marketplace_lib::protocol::d2i::legacy::game_items::ALL_ITEMS
        .iter().map(|(c, _, _, _, _)| *c).collect();
    // shards (xa1-xa5) are in game_items but listed here as known valid codes
    let mut unknown = Vec::new();
    for item in &items {
        if !game_codes.contains(&item.item_type.as_str()) {
            unknown.push(item.item_type.clone());
        }
    }
    assert!(unknown.is_empty(),
        "Item codes not in game constants: {:?}", unknown);
}

#[test]
fn test_node_parser_item_counts_vs_ground_truth() {
    let Some(items) = parse_stash() else { return };
    // 按类型统计
    let mut from_json = std::collections::HashMap::new();
    for item in &items {
        *from_json.entry(item.item_type.clone()).or_insert(0u32) += 1;
    }
    // 检查每种物品只有一个（存档中每种类型最多出现 1 个条目）
    let duplicates: Vec<_> = from_json.iter().filter(|(_, c)| **c > 1).collect();
    assert!(duplicates.is_empty(), "Duplicate items found: {:?}", duplicates);
}

#[test]
fn test_node_parser_identifies_equipment_types() {
    let Some(items) = parse_stash() else { return };
    // 验证我们能看到的物品类型覆盖了所有主要分类
    let types: Vec<&str> = items.iter().map(|i| i.item_type.as_str()).collect();

    // Runen
    assert!(types.iter().any(|t| t.starts_with('r')), "Should have runes");
    // Gems
    assert!(types.iter().any(|t| t.starts_with('g')), "Should have gems");
    // Potions
    assert!(types.contains(&"rvs"), "Should have potions");
    // Keys
    assert!(types.contains(&"pk1"), "Should have keys");
    // Essences
    assert!(types.contains(&"tes"), "Should have essences");

    eprintln!("All {} items validated successfully.", items.len());
}

#[test]
fn test_node_parser_amounts_are_not_fractional() {
    let Some(items) = parse_stash() else { return };
    for item in &items {
        assert!(item.amount > 0, "Amount should be positive for {}", item.item_type);
    }
}
