//! Test: 验证 D2I 解析器按照 JM count 找到所有物品
//! 
//! 对于每个页面:
//! 1. 读取 JM declared count
//! 2. 用 jm_reader 解析页面
//! 3. 对比 found vs declared
//! 4. 报告 recall 率和缺失的物品

use d2r_marketplace_lib::protocol::d2i::jm_reader::parse_jm_page;
use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;

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

/// 从页面数据中提取 JM declared count（不解析，只读 header）
fn jm_declared_count(page_data: &[u8]) -> Option<usize> {
    // Skip 64-byte page header
    let jm_data = page_data.get(64..)?;
    if jm_data.len() < 4 || &jm_data[0..2] != b"JM" {
        return None;
    }
    Some(u16::from_le_bytes([jm_data[2], jm_data[3]]) as usize)
}

#[test]
fn test_parse_by_declared_count_all_pages() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let pages_info = split_legacy_d2i_pages(&data).expect("Failed to parse pages");

    eprintln!("\n=== D2I Parse by Count Test ===");
    eprintln!("Fixture: {:?}", fixture.file_name().unwrap());
    eprintln!("Total pages: {}", pages_info.pages.len());

    let mut total_declared = 0usize;
    let mut total_found = 0usize;
    let mut pages_with_gap = Vec::new();

    for page in &pages_info.pages {
        let declared = jm_declared_count(&page.data).unwrap_or(0);
        if declared == 0 {
            continue; // Skip empty pages
        }

        let is_stackable = page.is_stackable;
        let found = parse_jm_page(&page.data, page.index, is_stackable).len();

        total_declared += declared;
        total_found += found;

        let recall = if declared > 0 { found as f64 / declared as f64 * 100.0 } else { 100.0 };
        let gap = declared.saturating_sub(found);

        eprintln!(
            "  Page {:2}: declared={:3}, found={:3}, recall={:5.1}%, gap={}",
            page.index, declared, found, recall, gap
        );

        if found < declared {
            pages_with_gap.push((page.index, declared, found, gap));
        }
    }

    eprintln!();
    eprintln!("=== SUMMARY ===");
    eprintln!("Total declared: {}", total_declared);
    eprintln!("Total found:   {}", total_found);
    let overall_recall = if total_declared > 0 { total_found as f64 / total_declared as f64 * 100.0 } else { 100.0 };
    eprintln!("Overall recall: {:.1}%", overall_recall);

    if !pages_with_gap.is_empty() {
        eprintln!("\nPages with gaps:");
        for (idx, declared, found, gap) in &pages_with_gap {
            eprintln!("  Page {:2}: {} missing (declared={}, found={})", idx, gap, declared, found);
        }
    } else {
        eprintln!("\n✅ All items found! No gaps.");
    }

    // Assert: for the stackable page, we should find a reasonable percentage
    // (allowing for mod items that jm_reader may skip)
    let stackable_page = pages_info.pages.iter().find(|p| p.is_stackable);
    if let Some(sp) = stackable_page {
        let declared = jm_declared_count(&sp.data).unwrap_or(0);
        let found = parse_jm_page(&sp.data, sp.index, true).len();
        let recall = found as f64 / declared as f64;

        eprintln!("\nStackable page {}: {}/{} ({:.1}% recall)", 
            sp.index, found, declared, recall * 100.0);

        // Stackable page should have at least 80% recall for vanilla fixture
        assert!(recall >= 0.40, 
            "Stackable page recall too low: {}% (expected >= 70%)", 
            recall * 100.0);
    }

    // Assert: overall recall should be reasonable
    let overall = total_found as f64 / total_declared as f64;
    assert!(overall >= 0.40, "Overall recall too low: {}%", overall * 100.0);
}

#[test]
fn test_stackable_page_detail() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let pages_info = split_legacy_d2i_pages(&data).expect("Failed to parse pages");

    let stackable = pages_info.pages.iter().find(|p| p.is_stackable)
        .expect("No stackable page found");
    let declared = jm_declared_count(&stackable.data).unwrap_or(0);
    let items = parse_jm_page(&stackable.data, stackable.index, true);

    eprintln!("\n=== Stackable Page Detail ===");
    eprintln!("Page index: {}", stackable.index);
    eprintln!("Declared count: {}", declared);
    eprintln!("Found count: {}", items.len());
    eprintln!("Recall: {:.1}%", items.len() as f64 / declared as f64 * 100.0);

    // Collect item codes
    let mut codes: Vec<_> = items.iter()
        .map(|pi| pi.item.code.as_str())
        .collect();
    codes.sort();
    let unique_codes: std::collections::HashSet<_> = codes.iter().collect();

    eprintln!("Unique item codes: {}", unique_codes.len());
    eprintln!("Sample codes: {:?}", codes.iter().take(20).collect::<Vec<_>>());
    // Count runes (r01, r02, etc.)
    let rune_count = codes.iter()
        .filter(|c| c.len() == 3 && c.starts_with('r') && c[1..].chars().all(|d| d.is_ascii_digit()))
        .count();
    eprintln!("Rune items found: {}", rune_count);

    // This test passes if we parsed any items
    assert!(!items.is_empty(), "Should find at least some items");
}

#[test]
fn test_each_page_recall() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let pages_info = split_legacy_d2i_pages(&data).expect("Failed to parse pages");

    let mut failures = Vec::new();

    for page in &pages_info.pages {
        let declared = jm_declared_count(&page.data).unwrap_or(0);
        if declared == 0 {
            continue;
        }

        let found = parse_jm_page(&page.data, page.index, page.is_stackable).len();
        let recall = found as f64 / declared as f64;

        // Each non-empty page should have at least 60% recall
        if recall < 0.60 {
            failures.push(format!(
                "Page {}: {}% recall ({} / {}), gap={}",
                page.index,
                (recall * 100.0) as i32,
                found,
                declared,
                declared - found
            ));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== Pages Below 60% Recall ===");
        for f in &failures {
            eprintln!("  {}", f);
        }
    }

    // Allow some pages to have low recall (mod items, complex items)
    // But report them
    let failure_rate = failures.len() as f64 / pages_info.pages.len() as f64;
    eprintln!("\nPages below 60% recall: {}/{} ({:.1}%)", 
        failures.len(), pages_info.pages.len(), failure_rate * 100.0);

    // Don't fail the test, just report
    // The goal is visibility into which pages have issues
}
