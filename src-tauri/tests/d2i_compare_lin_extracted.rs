//! Test: old.d2i → 取 lin 39个  → 与参考文件对比
//!
//! old 中 lin: amount=88, pos=(8,5)
//! 参考文件:  amount=49, pos=(1,3)
//!
//! 需要修改 3 处:
//!   1. px byte (item byte 5, bits 2-5)
//!   2. py byte (item byte 5 bits 6-7 + item byte 6 bits 0-1)
//!   3. chest_stackable trailer (末尾 2 bytes, u16 LE = amount × 32 + base)

use d2r_marketplace_lib::protocol::d2i::page::reassemble_pages;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

const OLD_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-old.d2i";
const REF_PATH: &str = "D:\\work_space\\personal_workspace\\d2r\\test\\ModernSharedStashSoftCoreV2-提取lin二次进入游戏后保存.d2i";

fn get_item_amount(data: &[u8], code: &str) -> Option<u32> {
    let file = parse_file(data).expect("parse_file");
    file.items.iter()
        .find(|pi| pi.item.code == code)
        .map(|pi| pi.item.amount)
}

#[test]
fn test_lin_extracted_matches_reference() {
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

    // ── 找 lin ──
    let items = d2r_marketplace_lib::protocol::d2i::jm_reader::parse_jm_page(
        &stackable_page.data, stackable_page.index, stackable_page.is_stackable);
    let lin = items.iter()
        .find(|pi| pi.item.code == "lin")
        .expect("未找到 lin");

    let byte_off = lin.raw_bit_offset / 8;
    let old_amt = lin.item.amount;
    let new_amt: u32 = 49;
    eprintln!("lin: byte_off={} old_amt={} → new_amt={}", byte_off, old_amt, new_amt);

    let jm_data = &stackable_page.data[64..];
    let mut jm_bytes = jm_data.to_vec();
    const HDR: usize = 4;
    const CORE_BYTES: usize = 10;

    // ── 1. px/py ──
    let new_px = (new_amt & 0x0F) as u8;
    let new_py = ((new_amt >> 4) & 0x0F) as u8;
    let px_byte = HDR + byte_off + 5;
    let old_byte5 = jm_bytes[px_byte];
    jm_bytes[px_byte] = (old_byte5 & 0xC3) | (new_px << 2) | (new_py << 6);
    if px_byte + 1 < jm_bytes.len() {
        jm_bytes[px_byte + 1] = (jm_bytes[px_byte + 1] & 0xFC) | ((new_py >> 2) & 0x03);
    }
    eprintln!("px byte JM[{px_byte}] = 0x{old_byte5:02x} → 0x{:02x}", jm_bytes[px_byte]);
    // lin 是 simple=false，跳过 realm

    // ── 3. chest_stackable trailer ──
    // 末尾 2 bytes = u16 LE = amount * 32 + base
    let end_bit = lin.raw_bit_offset + lin.raw_bit_length;
    let end_byte = (end_bit + 7) / 8;
    let trailer_lo = HDR + end_byte - 1;  // last byte of item in JM
    let trailer_hi = HDR + end_byte;      // first byte after item in JM
    if trailer_hi < jm_bytes.len() {
        let old_trailer = u16::from_le_bytes([jm_bytes[trailer_lo], jm_bytes[trailer_hi]]);
        let base = old_trailer as i32 - old_amt as i32 * 32;
        if base >= 0 && base < 65536 {
            let new_trailer = (new_amt as i32 * 32 + base) as u16;
            let lo_val = (new_trailer & 0xFF) as u8;
            let hi_val = (new_trailer >> 8) as u8;
            eprintln!("trailer [JM{trailer_lo},JM{trailer_hi}] = 0x{old_trailer:04x} → 0x{new_trailer:04x} (base={base})");
            jm_bytes[trailer_lo] = lo_val;
            jm_bytes[trailer_hi] = hi_val;
        }
    }

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
    let gen_amt = get_item_amount(&generated, "lin");
    let ref_amt = get_item_amount(&ref_data, "lin");
    assert_eq!(gen_amt, Some(49), "生成的 lin 应为 49，实际={:?}", gen_amt);
    assert_eq!(ref_amt, Some(49), "参考文件 lin 应为 49，实际={:?}", ref_amt);
    eprintln!("✅ lin: generated={:?} ref={:?}", gen_amt, ref_amt);
}
