//! Test: old.d2i → 取光 r03 → 与参考文件对比
//!
//! 流程:
//!   1. 解析 old.d2i → 找到堆叠页
//!   2. 在 JM 流中修改 r03 的 position_x (px) + realm data
//!     - px:7→0 (amount = (py<<4)|px)
//!     - realm byte: 修正冗余 amount 编码
//!   3. 重拼 page data（64B header + 修改后的 JM）
//!   4. 逐字节对比参考文件（跳过 Page 6 已知差异）

use d2r_marketplace_lib::protocol::d2i::page::reassemble_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const OLD_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-old.d2i";
const REF_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-r03取光二次进入游戏后保存.d2i";

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

    // ── 2. 修改 JM 数据 ──
    // r03 在 JM payload 中: off=2360b len=80b → byte 295
    //   px: item bits 42-45 → JM payload byte 300, byte bits 2-5
    //   realm data: JM payload byte 320 (= hdr 4 + byte_off 295 + byte_len 10 + 15)
    //
    // 参考: 现有 test_deduct_three_runes_via_warehouse_deposit 中 realm 修正逻辑

    let jm_data = &stackable_page.data[64..]; // exclude 64B page header
    let mut jm_bytes = jm_data.to_vec();

    const HDR: usize = 4; // JM header "JM" + u16 count

    let byte_off = 2360 / 8;    // = 295
    let byte_len = (2360 + 80 + 7) / 8 - byte_off; // = 10
    let old_amt: u8 = 7;
    let new_amt: u8 = 0;

    // (a) position_x (bits 42-45) → 设置 amount=0
    let px_byte = HDR + byte_off + 5; // JM stream byte for px
    let old_px = jm_bytes[px_byte];
    jm_bytes[px_byte] = old_px & 0xC3; // clear bits 5-2 (px=0)
    eprintln!("px byte JM[{px_byte}] = 0x{old_px:02x} → 0x{:02x}", jm_bytes[px_byte]);

    // (b) realm data (amount 的冗余编码)
    let realm_off = HDR + byte_off + byte_len + 15;
    let realm_b = jm_bytes[realm_off];
    let step = [128u8, 64, 32, 16, 8, 4, 2, 1].into_iter()
        .find(|&s| (realm_b as u16) > (old_amt as u8 as u16) * (s as u16))
        .unwrap_or(1);
    let base = (realm_b as i16) - (old_amt as i16) * (step as i16);
    let new_realm = (base + (new_amt as i16) * (step as i16)) as u8;
    eprintln!("realm byte JM[{realm_off}] = 0x{realm_b:02x} → 0x{new_realm:02x} (step={step}, base={base})");
    jm_bytes[realm_off] = new_realm;

    // ── 3. 重拼 page data ──
    let mut page_data = stackable_page.data[..64].to_vec();
    page_data.extend_from_slice(&jm_bytes);
    let new_size = page_data.len() as u32;
    page_data[16..20].copy_from_slice(&new_size.to_le_bytes());
    eprintln!("新 page data size = {new_size}");

    // ── 4. 重拼完整文件 ──
    let mut updated_pages = old_file.pages.clone();
    for p in &mut updated_pages {
        if p.index == stackable_page.index {
            p.data = page_data;
            break;
        }
    }
    let generated = reassemble_pages(&updated_pages, &old_file.tail);

    // ── 5. 读取参考文件 ──
    if !std::path::Path::new(REF_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REF_PATH);
        return;
    }
    let ref_data = std::fs::read(REF_PATH).expect("读取参考文件失败");

    // ── 6. 逐字节对比（跳过 Page 6 已知差异）──
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

    // ── 7. 解析验证 r03=0，其他符文不变 ──
    let gen_amt = get_rune_amount(&generated, "r03");
    let ref_amt = get_rune_amount(&ref_data, "r03");
    assert_eq!(gen_amt, Some(0), "生成的 r03 应为 0，实际={:?}", gen_amt);
    assert_eq!(ref_amt, Some(0), "参考文件 r03 应为 0，实际={:?}", ref_amt);
    eprintln!("✅ r03: generated={:?} ref={:?}", gen_amt, ref_amt);

    for code in &["r09", "r10"] {
        let ga = get_rune_amount(&generated, code);
        let ra = get_rune_amount(&ref_data, code);
        assert_eq!(ga, ra, "{} 金额不一致: gen={:?} ref={:?}", code, ga, ra);
    }
    eprintln!("✅ r09/r10 金额一致");
}
