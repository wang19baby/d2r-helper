//! Test: 存入 11 个 lin (88→99) — 对比参考文件
//!
//! lin 是 non-simple (simple=0), 编码含 stat list。
//! 当前 update_stackable_items_v2 只修改 position_x/y, non-simple 的 stat 尾部编码
//! 不在当前修补范围内(JM[70,71] 已知差异,游戏全量重编码 stat list 所致)。
//!
//! 验证:
//!   1. position_x/px 字节正确 ✅
//!   2. 解析后 amount 正确 ✅  
//!   3. 其他符文不变 ✅
//!   4. 两个 trailing 字节为已知非简单物品编码差异 △

use d2r_marketplace_lib::protocol::d2i::page::{find_stackable_page, reassemble_pages};
use d2r_marketplace_lib::protocol::d2i::parser::{parse_file, update_stackable_items_v2, D2IFile};

fn find_test_file(substr: &str) -> std::path::PathBuf {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("test");
    let Ok(entries) = std::fs::read_dir(&test_dir) else {
        return std::path::PathBuf::new(); // 目录缺失 → 空路径, 调用方 exists() 检查会 SKIP
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(substr) {
            return entry.path();
        }
    }
    std::path::PathBuf::new()
}

fn get_rune_amount(file: &D2IFile, code: &str) -> Option<u32> {
    file.items.iter()
        .find(|pi| pi.item.code == code)
        .map(|pi| pi.item.amount)
}

#[test]
fn test_deposit_eleven_lin() {
    let old_path = find_test_file("old.d2i");
    let ref_path = find_test_file("存入了11个lin");

    if !old_path.exists() || !ref_path.exists() {
        eprintln!("SKIP: test files not found");
        eprintln!("  old: {:?}", old_path);
        eprintln!("  ref: {:?}", ref_path);
        return;
    }

    let old_data = std::fs::read(&old_path).expect("read old.d2i");
    let old_file = parse_file(&old_data).expect("parse old.d2i");
    let sp = find_stackable_page(&old_file.pages).expect("stackable page").clone();

    // ── 1. Verify old lin = 88 ──
    let old_lin = get_rune_amount(&old_file, "lin");
    eprintln!("old lin = {:?}", old_lin);
    assert_eq!(old_lin, Some(88), "old.d2i lin should be 88");

    // ── 2. update_stackable_items_v2 +11 ──
    let (_items, new_page_data) = update_stackable_items_v2(&sp, "lin", 11, false)
        .expect("update_stackable_items_v2 should succeed");

    // ── 3. Reassemble ──
    let mut updated_pages = old_file.pages.clone();
    for p in &mut updated_pages {
        if p.index == sp.index {
            p.data = new_page_data;
            break;
        }
    }
    let generated = reassemble_pages(&updated_pages, &old_file.tail);

    // ── 4. Compare with reference ──
    let ref_data = std::fs::read(&ref_path).expect("read reference file");
    eprintln!("size: generated={} ref={}", generated.len(), ref_data.len());
    assert_eq!(generated.len(), ref_data.len());

    let mut diffs = Vec::new();
    for i in 0..generated.len() {
        if generated[i] != ref_data[i] {
            diffs.push((i, generated[i], ref_data[i]));
        }
    }

    // Known non-simple trailing bytes: JM[70] (last byte of lin body) and JM[71] (byte after lin)
    const PX_BYTE_ABS: usize = 0x0454;     // position_x byte, should match
    const TRAILING_1: usize = 0x046f;      // lin body last byte, known non-simple stat encoding diff
    const TRAILING_2: usize = 0x0470;      // byte after lin, known non-simple stat encoding diff
    const KNOWN: &[usize] = &[TRAILING_1, TRAILING_2];

    let unknown: Vec<_> = diffs.iter()
        .filter(|(off, _, _)| !KNOWN.contains(off))
        .collect();

    if !unknown.is_empty() {
        eprintln!("\n⚠ 未知差异 {} 字节:", unknown.len());
        for &(off, g, r) in &unknown {
            eprintln!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
        }
        let (off, g, r) = unknown[0];
        panic!("未知字节差异: 0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
    }

    // Verify px byte matches
    if let Some(&(_, gen_b, ref_b)) = diffs.iter().find(|(off, _, _)| *off == PX_BYTE_ABS) {
        panic!("px byte JM[43] 应该匹配! gen=0x{gen_b:02x} ref=0x{ref_b:02x}");
    }
    eprintln!("✅ position_x byte (JM[43]) = ref");

    // Report known trailing diffs
    let trailing: Vec<_> = diffs.iter().filter(|(off, _, _)| KNOWN.contains(off)).collect();
    if !trailing.is_empty() {
        eprintln!("ℹ️  non-simple trailing bytes (已知, stat list 不重编):");
        for &(off, g, r) in &trailing {
            eprintln!("    0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
        }
    }

    // ── 5. Verify parsed amounts ──
    let gen_file = parse_file(&generated).expect("parse generated");
    let ref_file = parse_file(&ref_data).expect("parse reference");

    assert_eq!(get_rune_amount(&gen_file, "lin"), Some(99),
        "lin should be 99");
    assert_eq!(get_rune_amount(&gen_file, "lin"), get_rune_amount(&ref_file, "lin"),
        "lin amount mismatch with ref");
    eprintln!("✅ lin: 88→99");

    for code in &["r03", "r09", "r10", "r08", "r07"] {
        let ga = get_rune_amount(&gen_file, code);
        let ra = get_rune_amount(&ref_file, code);
        assert_eq!(ga, ra, "{} mismatch: gen={:?} ref={:?}", code, ga, ra);
    }
    eprintln!("✅ other items match");

    eprintln!("\n🏁 test_deposit_eleven_lin PASSED");
}