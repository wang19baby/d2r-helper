//! Test: 存入 4 个 r09 到堆叠高级页 — 对比参考文件
//!
//! 场景:
//!   1. 从 old.d2i 解析得到 r09 = 10 个
//!   2. 用 update_stackable_items_v2 +4 个 r09 → 14 个
//!   3. 重拼 pages → 逐字节对比参考文件
//!   4. 验证 amounts: r09=14, 其他不变

use d2r_marketplace_lib::protocol::d2i::page::{find_stackable_page, reassemble_pages};
use d2r_marketplace_lib::protocol::d2i::parser::{parse_file, update_stackable_items_v2, D2IFile};

fn test_file_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("test")
        .join(name)
}

fn get_rune_amount(file: &D2IFile, code: &str) -> Option<u32> {
    file.items.iter()
        .find(|pi| pi.item.code == code)
        .map(|pi| pi.item.amount)
}

/// 获取测试文件路径（中文文件名通配搜索）
fn find_test_file(substr: &str) -> std::path::PathBuf {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("test");
    let Ok(entries) = std::fs::read_dir(&test_dir) else {
        return test_file_path(substr); // 目录缺失 → 走 fallback
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(substr) {
            return entry.path();
        }
    }
    // fallback: try test_file_path directly
    test_file_path(substr)
}

#[test]
fn test_deposit_four_r09_via_update_stackable_items() {
    let old_path = find_test_file("old.d2i");
    let ref_path = find_test_file("存入了4个r09");

    if !old_path.exists() || !ref_path.exists() {
        eprintln!("SKIP: test files not found");
        eprintln!("  old: {:?}", old_path);
        eprintln!("  ref: {:?}", ref_path);
        return;
    }

    // ── 1. 解析 old.d2i ──
    let old_data = std::fs::read(&old_path).expect("read old.d2i");
    let old_file = parse_file(&old_data).expect("parse old.d2i");

    // ── 2. 找到堆叠页 ──
    let sp = find_stackable_page(&old_file.pages).expect("stackable page").clone();
    eprintln!("堆叠页 index={} off=0x{:04x} data.len={}",
        sp.index, sp.offset, sp.data.len());

    // ── 3. 验证初始 r09 = 10 ──
    assert_eq!(get_rune_amount(&old_file, "r09"), Some(10),
        "old.d2i 中 r09 应为 10");

    // ── 4. 调用 update_stackable_items_v2 存入 4 个 r09 ──
    let (updated_items, new_page_data) = update_stackable_items_v2(&sp, "r09", 4, false)
        .expect("update_stackable_items_v2 should succeed");

    // 验证解析结果
    let new_r09 = updated_items.iter()
        .find(|pi| pi.item.code == "r09")
        .expect("r09 must still exist");
    assert_eq!(new_r09.item.amount, 14,
        "r09 should be 14 after depositing 4 (was 10)");
    eprintln!("✅ r09: {} → {} (+4)", 10, new_r09.item.amount);

    // ── 5. 重拼 pages ──
    let mut updated_pages = old_file.pages.clone();
    for p in &mut updated_pages {
        if p.index == sp.index {
            p.data = new_page_data;
            break;
        }
    }
    let generated = reassemble_pages(&updated_pages, &old_file.tail);

    // ── 6. 读取参考文件 ──
    let ref_data = std::fs::read(&ref_path).expect("read reference file");
    eprintln!("文件大小: generated={} ref={}", generated.len(), ref_data.len());
    assert_eq!(generated.len(), ref_data.len(), "文件大小不一致");

    // ── 7. 逐字节对比 ──
    let mut diffs = Vec::new();
    for i in 0..generated.len() {
        if generated[i] != ref_data[i] {
            diffs.push((i, generated[i], ref_data[i]));
        }
    }

    if !diffs.is_empty() {
        eprintln!("\n⚠ 共 {} 字节差异:", diffs.len());
        for &(off, g, r) in &diffs {
            eprintln!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
        }
    }

    // ── 8. 断言关键字节无差异 ──
    // 注: Page 6 是 mod 页(stackable=2)，游戏保存时会重写元数据
    // 已知差异位置: 0x0dfe, 0x0e08 等 (参见 d2i_compare_r03_extracted.rs)
    // 如果只有 Page 6 差异则通过
    let page6_off: usize = 0x0d0a;
    let page6_size: usize = 538;
    let page6_end = page6_off + page6_size;

    let critical: Vec<_> = diffs.iter()
        .filter(|(off, _, _)| !(*off >= page6_off && *off < page6_end))
        .collect();

    assert!(critical.is_empty(),
        "堆叠页数据不一致！仍有 {} 个关键字节不同:\n{}",
        critical.len(),
        critical.iter()
            .take(20)
            .map(|(off, g, r)| format!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    if !diffs.is_empty() {
        eprintln!("⚠ 仅有 Page 6 已知差异 ({} 字节, 游戏重写元数据)，通过", diffs.len());
    } else {
        eprintln!("✅ 逐字节完全一致！");
    }

    // ── 9. 验证 amounts ──
    let gen_file = parse_file(&generated).expect("parse generated");
    let ref_file = parse_file(&ref_data).expect("parse reference");
    assert_eq!(get_rune_amount(&gen_file, "r09"), get_rune_amount(&ref_file, "r09"),
        "r09 amount mismatch");
    assert_eq!(get_rune_amount(&gen_file, "r09"), Some(14),
        "r09 must be exactly 14");
    eprintln!("✅ r09: generated={:?} ref={:?}",
        get_rune_amount(&gen_file, "r09"),
        get_rune_amount(&ref_file, "r09"));

    // 验证其他符文不变
    for code in &["r03", "r10", "r08", "r07"] {
        let ga = get_rune_amount(&gen_file, code);
        let ra = get_rune_amount(&ref_file, code);
        assert_eq!(ga, ra, "{} amount mismatch: gen={:?} ref={:?}", code, ga, ra);
    }
    eprintln!("✅ 其他符文金额一致");
}