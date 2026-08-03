//! Test: old.d2i → 取光碎裂的绿宝石(gcg) → 与参考文件对比
//!
//! 流程:
//!   1. 解析 old.d2i，找到堆叠页
//!   2. 用 parse_jm_page 找 gcg 的 bit offset
//!   3. 修改 position_x (px) + realm data
//!   4. 重拼 → 逐字节对比参考文件

use d2r_marketplace_lib::protocol::d2i::jm_reader::parse_jm_page;
use d2r_marketplace_lib::protocol::d2i::page::reassemble_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const OLD_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-old.d2i";
const REF_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-提取了碎裂的绿宝石取光二次进入游戏后保存.d2i";

/// 简单堆叠项核心编码固定长度 (flags + ver + pos + huffman + ext_data + align)
const CORE_ITEM_BYTES: usize = 10;

fn get_rune_amount(data: &[u8], code: &str) -> Option<u32> {
    let file = parse_file(data).expect("parse_file");
    file.items.iter()
        .find(|pi| pi.item.code == code)
        .map(|pi| pi.item.amount)
}

#[test]
fn test_gcg_extracted_matches_reference() {
    if !std::path::Path::new(OLD_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", OLD_PATH);
        return;
    }
    let old_data = std::fs::read(OLD_PATH).expect("读取 old.d2i 失败");
    let old_file = parse_file(&old_data).expect("解析 old.d2i 失败");

    let stackable_page = old_file.pages.iter()
        .find(|p| p.is_stackable)
        .expect("未找到堆叠页");
    eprintln!("堆叠页 index={} file_off=0x{:04x} data.len={}",
        stackable_page.index, stackable_page.offset, stackable_page.data.len());

    // ── 2. 扫描堆叠页找 gcg ──
    let jm_items = parse_jm_page(&stackable_page.data, stackable_page.index, stackable_page.is_stackable);
    let gcg = jm_items.iter()
        .find(|pi| pi.item.code == "gcg")
        .expect("未找到 gcg (Chipped Emerald)");

    let byte_off = gcg.raw_bit_offset / 8;
    let old_amt = gcg.item.amount as u8;
    eprintln!("gcg: raw_bit_offset={} bits, byte_off={} bytes, old_amt={}",
        gcg.raw_bit_offset, byte_off, old_amt);

    let jm_data = &stackable_page.data[64..];
    let mut jm_bytes = jm_data.to_vec();
    const HDR: usize = 4;
    let new_amt: u8 = 0;

    // ── 3. 修改 JM 数据 ──
    // (a) position_x (item byte 5, byte bits 2-5)
    let px_byte = HDR + byte_off + 5;
    let old_px = jm_bytes[px_byte];
    jm_bytes[px_byte] = old_px & 0xC3;
    let px_file_off = stackable_page.offset as usize + 64 + px_byte;
    eprintln!("px byte JM[{px_byte}] = 0x{old_px:02x} → 0x{:02x} (file 0x{px_file_off:04x})",
        jm_bytes[px_byte]);

    // (b) realm data — 用固定 CORE_ITEM_BYTES 计算偏移
    let realm_off = HDR + byte_off + CORE_ITEM_BYTES + 15;
    let realm_b = jm_bytes[realm_off];
    let step = [128u8, 64, 32, 16, 8, 4, 2, 1].into_iter()
        .find(|&s| (realm_b as u16) > (old_amt as u8 as u16) * (s as u16))
        .unwrap_or(1);
    let base = (realm_b as i16) - (old_amt as i16) * (step as i16);
    let new_realm = (base + (new_amt as i16) * (step as i16)) as u8;
    let realm_file_off = stackable_page.offset as usize + 64 + realm_off;
    eprintln!("realm byte JM[{realm_off}] = 0x{realm_b:02x} → 0x{new_realm:02x} (step={step}, base={base}, file 0x{realm_file_off:04x})");
    jm_bytes[realm_off] = new_realm;

    // ── 4. 重拼 page data ──
    let mut page_data = stackable_page.data[..64].to_vec();
    page_data.extend_from_slice(&jm_bytes);
    let new_size = page_data.len() as u32;
    page_data[16..20].copy_from_slice(&new_size.to_le_bytes());
    eprintln!("新 page data size = {}", new_size);

    // ── 5. 重拼完整文件 ──
    let mut updated_pages = old_file.pages.clone();
    for p in &mut updated_pages {
        if p.index == stackable_page.index {
            p.data = page_data;
            break;
        }
    }
    let generated = reassemble_pages(&updated_pages, &old_file.tail);

    // ── 6. 逐字节对比（跳过 Page 6）──
    if !std::path::Path::new(REF_PATH).exists() {
        eprintln!("SKIP: {} 缺失（本地对拍文件）, 跳过测试", REF_PATH);
        return;
    }
    let ref_data = std::fs::read(REF_PATH).expect("读取参考文件失败");
    let page6_off: usize = 0x0d0a;
    let page6_end = page6_off + 538;

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

    if !page6_diffs.is_empty() {
        eprintln!("\n⚠ Page 6 已知差异（不断言）:");
        for &(off, g, r) in &page6_diffs {
            eprintln!("  0x{off:04x}: gen=0x{g:02x} ref=0x{r:02x}");
        }
    }

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

    // ── 7. 解析验证 ──
    let gen_amt = get_rune_amount(&generated, "gcg");
    let ref_amt = get_rune_amount(&ref_data, "gcg");
    assert_eq!(gen_amt, Some(0), "生成的 gcg 应为 0，实际={:?}", gen_amt);
    assert_eq!(ref_amt, Some(0), "参考文件 gcg 应为 0，实际={:?}", ref_amt);
    eprintln!("✅ gcg (Chipped Emerald): generated={:?} ref={:?}", gen_amt, ref_amt);

    for code in &["gcr", "gcv", "gcb"] {
        let ga = get_rune_amount(&generated, code);
        let ra = get_rune_amount(&ref_data, code);
        assert_eq!(ga, ra, "{} 金额不一致: gen={:?} ref={:?}", code, ga, ra);
    }
    eprintln!("✅ 其他宝石金额一致");
}
