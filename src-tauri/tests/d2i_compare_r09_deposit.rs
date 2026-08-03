//! Test: old.d2i → 存入 4 个 r09 (10→14) → 与参考文件逐字节对比
//!
//! 流程:
//!   1. 解析 old.d2i → 找到堆叠页
//!   2. 调用 update_stackable_items_v2 +4 r09（修改 JM 中 px/py + realm data）
//!   3. 重拼 pages → 完整 d2i 文件
//!   4. 逐字节对比参考文件（跳过 Page 6 已知差异）
//!   5. 解析验证 r09=14，其他符文一致性

use d2r_marketplace_lib::protocol::d2i::page::reassemble_pages;
use d2r_marketplace_lib::protocol::d2i::parser::{parse_file, update_stackable_items_v2};

const OLD_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-old.d2i";
const REF_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-存入了4个r09二次进入游戏后保存.d2i";

fn get_rune_amount(data: &[u8], code: &str) -> Option<u32> {
    let file = parse_file(data).expect("parse_file");
    file.items.iter()
        .find(|pi| pi.item.code == code)
        .map(|pi| pi.item.amount)
}

#[test]
fn test_generated_matches_reference() {
    // ── 1. 读取 old.d2i ──
    if !std::path::Path::new(OLD_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", OLD_PATH);
        return;
    }
    let old_data = std::fs::read(OLD_PATH).expect("读取 old.d2i 失败");
    let old_file = parse_file(&old_data).expect("解析 old.d2i 失败");

    // 找到堆叠页 Page 5 (index=5, file_off=0x03e9)
    let stackable_page = old_file.pages.iter()
        .find(|p| p.is_stackable)
        .expect("未找到堆叠页");
    eprintln!("堆叠页 index={} file_off=0x{:04x} data.len={}",
        stackable_page.index, stackable_page.offset, stackable_page.data.len());

    // 验证 old 中 r09=10
    let old_r09 = get_rune_amount(&old_data, "r09");
    assert_eq!(old_r09, Some(10), "old.d2i r09 应为 10");
    eprintln!("✅ old.d2i r09 = {:?}", old_r09);

    // ── 2. 通过 update_stackable_items_v2 增加 4 个 r09 ──
    let (_items, new_page_data) = update_stackable_items_v2(
        stackable_page,
        "r09",
        4,               // delta = +4
        false,           // create_if_missing = false (r09 已存在)
    ).expect("update_stackable_items_v2 失败");

    // ── 3. 重拼完整文件 ──
    let mut updated_pages = old_file.pages.clone();
    for p in &mut updated_pages {
        if p.index == stackable_page.index {
            p.data = new_page_data;
            break;
        }
    }
    let generated = reassemble_pages(&updated_pages, &old_file.tail);

    // ── 4. 读取参考文件 ──
    if !std::path::Path::new(REF_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REF_PATH);
        return;
    }
    let ref_data = std::fs::read(REF_PATH).expect("读取参考文件失败");

    // ── 5. 逐字节对比（跳过 Page 6 已知差异）──
    // Page 6 (file_off=0x0d0a) 是 mod 页 (stackable=2)，游戏保存时会重写元数据
    // 已知差异位置: 0x0dfe, 0x0e08 (两个 u32 值 116↔120 互换)
    let page6_off: usize = 0x0d0a;
    let page6_size: usize = 538; // 0x21a
    let page6_end = page6_off + page6_size;

    eprintln!("\n文件大小: generated={} ref={}", generated.len(), ref_data.len());
    assert_eq!(generated.len(), ref_data.len(), "文件大小不一致");

    let mut page6_diffs = Vec::new();
    let mut critical_diffs = Vec::new();
    for i in 0..generated.len() {
        if generated[i] != ref_data[i] {
            if i >= page6_off && i < page6_end {
                page6_diffs.push((i, generated[i], ref_data[i]));
            } else {
                critical_diffs.push((i, generated[i], ref_data[i]));
            }
        }
    }

    // 报告 Page 6 差异但不断言
    if !page6_diffs.is_empty() {
        eprintln!("\n⚠ Page 6 已知差异（不断言，游戏保存时重写的元数据）:");
        for &(off, g, r) in &page6_diffs {
            eprintln!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
        }
    }

    // 关键区域必须完全一致
    assert!(critical_diffs.is_empty(),
        "堆叠页数据不一致！仍有 {} 个关键字节不同:\n{}",
        critical_diffs.len(),
        critical_diffs.iter()
            .take(20)
            .map(|&(off, g, r)| format!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    eprintln!("✅ 堆叠页数据逐字节一致！");

    // ── 6. 解析验证 r09=14，其他符文不变 ──
    let gen_r09 = get_rune_amount(&generated, "r09");
    let ref_r09 = get_rune_amount(&ref_data, "r09");
    assert_eq!(gen_r09, Some(14), "生成的 r09 应为 14，实际={:?}", gen_r09);
    assert_eq!(ref_r09, Some(14), "参考文件 r09 应为 14，实际={:?}", ref_r09);
    eprintln!("✅ r09: generated={:?} ref={:?}", gen_r09, ref_r09);

    for code in &["r01", "r03", "r10", "r13", "r21", "r33"] {
        let ga = get_rune_amount(&generated, code);
        let ra = get_rune_amount(&ref_data, code);
        assert_eq!(ga, ra, "{} 金额不一致: gen={:?} ref={:?}", code, ga, ra);
    }
    eprintln!("✅ 所有符文金额与参考文件一致");
}
