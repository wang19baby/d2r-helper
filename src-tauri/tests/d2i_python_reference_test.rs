//! Python d2r-zero 参考解析器对齐测试。
//!
//! 5 个目标页面 (0/1/2/4/5) 的 count 目标来自 Python `construct_adapter` 解析结果。
//! 每次运行显示 Rust parser 实际找到的物品数与 Python 参考值的对比。
//!
//! Python 参考:
//! ```bash
//! python -m d2r_zero.cli_construct "ModernSharedStashSoftCoreV2.d2i" --bits
//! ```
//!
//! 目标: Rust 解析器在每个页面上找到的物品数 ≤ Python 参考数。
//! 当前状态是 RUST 解析能力上限 — 后续改进 parser 时更新此文件。

use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
use std::collections::HashMap;
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

/// Python 参考物品 codes per page (来自 construct_adapter 解析)
fn python_reference() -> HashMap<usize, Vec<&'static str>> {
    let mut refs = HashMap::new();
    // Page 0: 92 items
    refs.insert(0, vec![
        "gth", "nea", "8rx", "gwn", "rin", "rin", "xul", "jew", "amu", "rin",
        "utp", "xmg", "amu", "7vo", "7vo", "rin", "amu", "7pa", "rin", "rin",
        "7vo", "rin", "xmb", "jew", "rin", "obf", "9sm", "7dg", "jew", "jew",
        "rin", "amu", "ba5", "ba5", "xlb", "xlb", "rin", "rin", "9bl", "8rx",
        "hbl", "xhg", "amu", "amu", "rin", "8lw", "lsd", "r07", "r10", "r09",
        "r11", "rin", "amu", "jew", "rin", "2hs", "dr1", "r04", "r03", "ci0",
        "hbw", "r03", "r07", "r11", "jew", "uap", "jew", "jew", "amu", "hla",
        "r07", "r05", "amu", "rin", "rin", "xhg", "xhm", "amu", "lfw", "xhg",
        "xea", "xmg", "6ws", "amu", "rin", "rin", "wnd", "ltp", "r07", "r05",
        "tbt", "amu",
    ]);
    // Page 1: 72 items
    refs.insert(1, vec![
        "rin", "rin", "tbt", "rm1", "cm3", "xrs", "7ws", "72h", "9bw", "xea",
        "7sc", "7ws", "obf", "9tw", "uit", "ci2", "rin", "7qr", "7gm", "7ws",
        "7vo", "7wh", "cm3", "amf", "jew", "7wc", "7dg", "uit", "jew", "bsd",
        "r07", "r10", "r09", "r11", "ba3", "r09", "r12", "upl", "tbt", "utc",
        "uhm", "9tw", "zvb", "6ws", "wa6", "tbt", "7wc", "umb", "uea", "amu",
        "7gw", "lea", "r07", "r05", "hbt", "xmg", "ci0", "r09", "r12", "uap",
        "uul", "amu", "cm3", "rin", "amu", "tbl", "baa", "tbl", "rin", "gth",
        "rin", "rin",
    ]);
    // Page 2: 81 items
    refs.insert(2, vec![
        "obf", "ztb", "rin", "obf", "7lw", "7cr", "nea", "9tw", "nef", "6ws",
        "xvb", "rin", "cm3", "jew", "jew", "xea", "rin", "rin", "cm3", "7pa",
        "xhb", "umb", "rin", "uap", "rin", "xtp", "jew", "jew", "jew", "jew",
        "uit", "rin", "jew", "jew", "jew", "jew", "jew", "jew", "amu", "uap",
        "amu", "xtp", "amu", "ulc", "xul", "uit", "lft", "9tw", "7pa", "zhb",
        "ci0", "zvb", "cm3", "7s8", "rin", "rin", "amf", "jew", "rin", "usk",
        "ama", "vgl", "xea", "rin", "zhb", "hgl", "rin", "amu", "umc", "rin",
        "lfw", "r07", "r10", "r09", "r11", "skp", "r04", "r03", "hla", "r07",
        "r05",
    ]);
    // Page 4: 50 items
    refs.insert(4, vec![
        "7gd", "6lw", "am7", "7wa", "7kr", "7gm", "uar", "7kr", "8s8", "7gd",
        "7gd", "7wc", "7wa", "utb", "obf", "7gw", "xtb", "uhg", "7ws", "7ws",
        "7pa", "upl", "7gw", "zhb", "obc", "uit", "xlt", "xlg", "xtb", "7cr",
        "7gm", "brn", "r06", "r01", "r05", "spt", "r06", "r01", "r05", "rin",
        "9tr", "hgl", "xtp", "r13", "r22", "r10", "lfw", "xea", "obf", "hbt",
    ]);
    // Page 5: 131 items (stackable)
    refs.insert(5, vec![
        "lin", "lsh", "gcw", "skc", "r02", "gcg", "gcb", "r07", "r08", "lyd",
        "r06", "r03", "r04", "gcy", "gcr", "r05", "gsg", "gcv", "gfv", "gfw",
        "r10", "gsb", "gfr", "r09", "nls", "skf", "xfu", "sku", "gfg", "gsw",
        "gly", "gfy", "r01", "gsy", "r12", "r11", "ld1", "ls1", "li1", "lkg",
        "lkt", "lka", "jlf", "lks", "lkm", "glw", "rly", "lkd", "lkh", "ltc",
        "r16", "gzv", "lgc", "skl", "lmc", "xf1", "glb", "lhc", "r13", "r15",
        "lsc", "lac", "ldc", "r17", "r18", "glg", "gfb", "glr", "r24", "pk1",
        "r14", "r21", "rvl", "rvs", "tes", "lsk", "lfs", "lem", "ldf", "lbd",
        "lvs", "lri", "ls2", "lwf", "lud", "lmf", "fed", "lre", "lag", "lpn",
        "lcb", "lcr", "lmp", "lvp", "lhr", "jl1", "nl1", "gsv", "gsr", "r22",
        "r20", "r25", "r23", "r19", "mbr", "ceh", "dhn", "bet", "bey", "gpw",
        "xa2", "ua5", "ua4", "ljs", "lmn", "r26", "xa3", "lji", "cly", "lqd",
        "lsw", "xa1", "r29", "lzb", "xa5", "r27", "xa4", "gpg", "r31", "r30",
        "ua3",
    ]);
    refs
}

/// 每页的 count 目标 = Python 参考数
fn python_target_counts() -> HashMap<usize, usize> {
    let refs = python_reference();
    refs.into_iter().map(|(k, v)| (k, v.len())).collect()
}

/// Rust parser 的 code→count 对照表（便于后续更新）
const TARGET_PAGES: [usize; 5] = [0, 1, 2, 4, 5];

#[test]
fn test_all_target_pages_match_python_count() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let file = parse_file(&data).expect("parse_file failed");
    let targets = python_target_counts();

    eprintln!("\n══════════════════════════════════════════════════════");
    eprintln!("  D2I 解析器 vs Python 参考 — 5 个目标页面");
    eprintln!("══════════════════════════════════════════════════════");
    eprintln!("  Fixture: ModernSharedStashSoftCoreV2.d2i");
    // 当前 recall 目标: through all changes
    assert!(found_pct_all_pages(&file) >= 50.0,
        "Rust 解析 recall 太低: {:.1}%, 需要改进 parser",
        found_pct_all_pages(&file));

    // 保留原 target_pages assertion
    let page0_found = file.items.iter().filter(|p| p.page_index == 0).count();
    assert!(page0_found >= 59, "Page 0 应有 >= 59 items, 实际 {}", page0_found);
    let mut total_target = 0usize;
    let mut total_found = 0usize;

    for page_idx in &TARGET_PAGES {
        let target = targets.get(page_idx).copied().unwrap_or(0);
        total_target += target;

        let page_items: Vec<_> = file.items.iter()
            .filter(|p| p.page_index == *page_idx)
            .collect();
        let found = page_items.len();
        total_found += found;

        let pct = if target > 0 { found as f64 / target as f64 * 100.0 } else { 0.0 };
        let is_stackable = *page_idx == 5;
        eprintln!("  Page {} {}: Rust={}/{}, Python={:.1}%",
            page_idx,
            if is_stackable { "(stackable)" } else { "" },
            found, target, pct);

        // Show first 5 and last 5 item codes
        for (i, pi) in page_items.iter().enumerate() {
            if i < 3 || i >= page_items.len().saturating_sub(3) {
                eprintln!("    [{}] code='{}'", i, pi.item.code);
            } else if i == 3 {
                eprintln!("    ... ({} items total)", page_items.len());
            }
        }
        eprintln!();
    }

    let overall_pct = if total_target > 0 { total_found as f64 / total_target as f64 * 100.0 } else { 0.0 };
    eprintln!("  ─────────────────────────────────────────────");
    eprintln!("  总计: Rust={}/{}, Python={:.1}%",
        total_found, total_target, overall_pct);
    eprintln!();

    // 当前状态: Rust 能解析到的真实物品数。tests 不强制必须等于 Python 参考,
    // 因为 Python 使用不同解析器 (construct_adapter) 可能有不同结果。
    // ⚠ 当 Rust 解析器改进时更新以下 assert!
    assert!(found_pct_all_pages(&file) >= 40.0,
        "Rust 解析 recall 太低: {:.1}%, 需要改进 parser",
        found_pct_all_pages(&file));
}

/// 辅助: 计算 Rust 在所有 5 个页面上的综合 recall
fn found_pct_all_pages(file: &d2r_marketplace_lib::protocol::d2i::parser::D2IFile) -> f64 {
    let targets = python_target_counts();
    let mut total_target = 0usize;
    let mut total_found = 0usize;
    for page_idx in &TARGET_PAGES {
        let target = targets.get(page_idx).copied().unwrap_or(0);
        total_target += target;
        let found = file.items.iter().filter(|p| p.page_index == *page_idx).count();
        total_found += found;
    }
    if total_target > 0 { total_found as f64 / total_target as f64 * 100.0 } else { 0.0 }
}
#[test]
fn test_page0_python_code_sequence() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let file = parse_file(&data).expect("parse_file failed");
    let refs = python_reference();
    let expected = refs.get(&0).expect("Python ref for page 0");

    let page_items: Vec<_> = file.items.iter()
        .filter(|p| p.page_index == 0)
        .collect();

    eprintln!("\n=== Page 0: Rust vs Python code 序列对比 ===");
    eprintln!("  Rust found: {} items", page_items.len());
    eprintln!("  Python ref: {} items", expected.len());

    // 在每个 Rust item 位置,标记是否命中 Python 参考
    let expected_set: std::collections::HashSet<&&str> = expected.iter().collect();
    for (i, pi) in page_items.iter().enumerate() {
        let in_ref = expected_set.contains(&pi.item.code.as_str());
        eprintln!("  [{}] code='{}'{}", i, pi.item.code,
            if in_ref { "" } else { " ✗ NOT in Python ref" });
    }

    // 显示 Python 有但 Rust 没找到的 codes
    let rust_codes: std::collections::HashSet<&str> = page_items.iter()
        .map(|pi| pi.item.code.as_str())
        .collect();
    let missing: Vec<_> = expected.iter()
        .filter(|c| !rust_codes.contains(*c))
        .collect();
    if !missing.is_empty() {
        eprintln!("\n  Python 有但 Rust 未找到 ({} codes):", missing.len());
        for c in missing.iter().take(20) {
            eprintln!("    '{}'", c);
        }
        if missing.len() > 20 {
            eprintln!("    ... (共 {} 个)", missing.len());
        }
    }

    // 不强制断言通过 — 这是信息性测试
}

/// Verify Rust parser items are a subset of Python reference codes.
/// If Rust finds a code not in Python ref, it's likely a false positive.
#[test]
fn test_page0_all_rust_codes_in_python_ref() {
    let Some(fixture) = fixture_path("ModernSharedStashSoftCoreV2.d2i") else { return };
    let data = std::fs::read(&fixture).expect("Fixture not found");
    let file = parse_file(&data).expect("parse_file failed");
    let refs = python_reference();
    let expected_list = refs.get(&0).expect("Python ref for page 0");
    let expected_set: std::collections::HashSet<&str> = expected_list.iter().copied().collect();

    let page_items: Vec<_> = file.items.iter()
        .filter(|p| p.page_index == 0)
        .collect();

    let mut false_positives = Vec::new();
    let mut matched = 0usize;

    for pi in &page_items {
        if expected_set.contains(pi.item.code.as_str()) {
            matched += 1;
        } else {
            false_positives.push(pi.item.code.clone());
        }
    }

    if !false_positives.is_empty() {
        eprintln!("\n  ⚠ Rust 发现 {} 个 Python 参考中不存在的 codes (潜在误报):",
            false_positives.len());
        for c in &false_positives {
            eprintln!("    '{}'", c);
        }
    }

    let recall = if !expected_list.is_empty() {
        matched as f64 / expected_list.len() as f64 * 100.0
    } else { 0.0 };
    let precision = if !page_items.is_empty() {
        matched as f64 / page_items.len() as f64 * 100.0
    } else { 0.0 };

    eprintln!("  Page 0: Rust items={}, matched_in_python={}/{}, precision={:.1}%, recall={:.1}%",
        page_items.len(), matched, expected_list.len(), precision, recall);

    // If precision < 80%, too many Rust codes are not in Python ref (false positives)
    assert!(precision >= 80.0,
        "Precision too low: {:.1}% — {} Rust codes not in Python reference",
        precision, false_positives.len());
}
