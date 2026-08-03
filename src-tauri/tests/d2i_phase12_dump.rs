#[cfg(test)]
mod dump_phase12 {
    use d2r_marketplace_lib::protocol::d2i::parser::parse_file;
    use d2r_marketplace_lib::core::BitReader;

    /// Hex dump first 100 bytes of a page's item data
    fn hex_dump_page_item_data(page: &d2r_marketplace_lib::protocol::d2i::page::Page, label: &str) {
        let item_data = page.item_bytes();
        let dump_len = item_data.len().min(100);
        eprintln!("\n--- {} item_data hex ({}/{}) ---", label, dump_len, item_data.len());
        for chunk in item_data[..dump_len].chunks(16) {
            let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
            eprintln!("  {}", hex.join(" "));
        }
    }

    /// Manual decode of item[0] at bit offset
    fn debug_item0_raw(page: &d2r_marketplace_lib::protocol::d2i::page::Page) {
        let item_data = page.item_bytes();
        let mut r = BitReader::new(item_data);
        let start_bit = 32; // after JM (16b) + count (16b)

        eprintln!("\n--- Manual trace of item[0] compact header (bit 32+) ---");
        r.seek(start_bit);
        let f0_3 = r.read_bit_array(4);
        let ident = r.read_bit();
        let f5_10 = r.read_bit_array(6);
        let sock = r.read_bit();
        let _f12 = r.read_bit();
        let _newf = r.read_bit();
        let _f14_15 = r.read_bit_array(2);
        let _is_ear = r.read_bit();
        let _starter = r.read_bit();
        let _f18_20 = r.read_bit_array(3);
        let si = r.read_bit();
        let eth = r.read_bit();
        let _f23 = r.read_bit();
        let _pers = r.read_bit();
        let _f25 = r.read_bit();
        let _grw = r.read_bit();
        let _f27_31 = r.read_bit_array(5);

        eprintln!("  flags hex: {:04x} (simple_item={}, ident={}, socketed={})",
            (f0_3[0] as u16) | ((f5_10[0] as u16) << 4) | ((sock as u16) << 10) | ((si as u16) << 4) | ((eth as u16) << 5),
            si, ident, sock);

        let ver = r.read_u8(3);
        let mode = r.read_u8(3);
        let loc = r.read_u8(4);
        let x = r.read_u8(4);
        let y = r.read_u8(4);
        let page = r.read_u8(3);
        let code = d2r_marketplace_lib::core::encoding::decode_huffman_string(&mut r);

        eprintln!("  ver={} mode={} loc={} x={} y={} page={}", ver, mode, loc, x, y, page);
        eprintln!("  code='{}' (offset={}b, code+3b={}b)", code, r.offset(),
            if si==1 { r.offset() + 4 } else { r.offset() + 6 });

        let ns = if si==1 { r.read_u8(1) } else { r.read_u8(3) };
        eprintln!("  num_sockets_hint={} (offset={}b)", ns, r.offset());
    }

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
    fn dump_hex_page0() {
        let path = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
        let data = std::fs::read(&path).unwrap();
        let file = parse_file(&data).unwrap();
        let page = &file.pages[0];
        hex_dump_page_item_data(page, "Page[0]");
        debug_item0_raw(page);
    }

    /// Debug item at a specific offset on page 0 (especially (1,0) items)
    /// v3 scan 后: 此测试依赖的 `w  s` (mod item with space) 被 ALL_ITEMS
    /// 验证拒绝,所以 ignore 该测试。
    #[test]
    #[ignore = "v3 scan rejects mod items with spaces; not regression test"]
    fn debug_item_at_offset() {
        use d2r_marketplace_lib::protocol::d2i::legacy::bit_reader::BitReader;
        use d2r_marketplace_lib::protocol::d2i::legacy::huffman::decode_huffman_string;
        let path = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
        let data = std::fs::read(&path).unwrap();
        let file = parse_file(&data).unwrap();
        let page = &file.pages[0];
        let raw = page.item_bytes();

        // Find the first (1,0) item with a bad code (contains spaces)
        let target = file.items.iter().find(|i|
            i.page_index == 0 && i.item.x == 1 && i.item.y == 0
            && (i.item.code.contains(' ') || i.item.code.len() != 3)
        ).unwrap();
        let bo = target.raw_bit_offset;
        eprintln!("\n=== Diagnosing item at offset={}b (item code='{}') ===", bo, target.item.code);

        // Legacy-style decode of compact header at this offset
        let mut r = BitReader::new(raw);
        r.seek(bo);

        let _b0_3 = r.read_bit_array(4);
        let ident = r.read_bit();
        let _b5_10 = r.read_bit_array(6);
        let sock = r.read_bit();
        let _b12 = r.read_bit_array(1);
        let _new = r.read_bit();
        let _b14_15 = r.read_bit_array(2);
        let _ear = r.read_bit();
        let _start = r.read_bit();
        let _b18_20 = r.read_bit_array(3);
        let si = r.read_bit(); // simple_item
        let eth = r.read_bit();
        let _b23 = r.read_bit_array(1);
        let pers = r.read_bit();
        let _b25 = r.read_bit_array(1);
        let rw = r.read_bit();
        let _b27_31 = r.read_bit_array(5);
        // total: 32b

        let ver = r.read_u16(3);
        let loc_id = r.read_u8(3);  // mode
        let eq_id = r.read_u8(4);   // location
        let px = r.read_u8(4);
        let py = r.read_u8(4);
        let alt_pos = r.read_u8(3); // alt_position_id
        let code = decode_huffman_string(&mut r);
        let ns = if si == 1 { r.read_u8(1) } else { r.read_u8(3) };

        eprintln!("  Legacy decode:");
        eprintln!("    simple_item={} identified={} socketed={} ethereal={} pers={} rw={}", si, ident, sock, eth, pers, rw);
        eprintln!("    ver={} mode={} loc={} x={} y={} alt_pos={} code='{}' ns={}", ver, loc_id, eq_id, px, py, alt_pos, code.trim(), ns);
        eprintln!("    after compact offset={}b", r.offset());
        eprintln!();

        // New read_compact decode
        let mut r2 = d2r_marketplace_lib::core::BitReader::new(raw);
        r2.seek(bo);
        let flags = d2r_marketplace_lib::protocol::common::ItemFlags::read(&mut r2).unwrap();
        let v = r2.read_u8(3);
        let mode = d2r_marketplace_lib::protocol::common::ItemMode::read(&mut r2).unwrap();
        let location = d2r_marketplace_lib::protocol::common::ItemLocation::read(&mut r2).unwrap();
        let x = r2.read_u8(4);
        let y = r2.read_u8(4);
        let page = d2r_marketplace_lib::protocol::common::ItemPage::read(&mut r2).unwrap();
        let code2 = d2r_marketplace_lib::core::encoding::decode_huffman_string(&mut r2);
        let ns2 = if flags.simple_item() { r2.read_u8(1) } else { r2.read_u8(3) };

        eprintln!("  New decode:");
        eprintln!("    raw_flags={:#010b} simple_item={} identified={} socketed={}", flags.raw, flags.simple_item(), flags.identified(), flags.socketed());
        eprintln!("    ver={} mode={:?} loc={:?} x={} y={} page={:?} code='{}' ns={}", v, mode, location, x, y, page, code2.trim(), ns2);
        eprintln!("    after compact offset={}b", r2.offset());

        // Compare bit positions
        eprintln!("\n  Legacy end at {}b, New end at {}b — diff={}b", r.offset(), r2.offset(), r.offset() as i64 - r2.offset() as i64);
    }

    #[test]
    fn dump_pages_0_to_5() {
        let path = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
        let data = std::fs::read(&path).unwrap();
        let file = parse_file(&data).unwrap();

        for page_idx in 0..=5 {
            if page_idx >= file.pages.len() { break; }
            let page = &file.pages[page_idx];
            let items: Vec<_> = file.items.iter().filter(|i| i.page_index == page_idx).collect();
            eprintln!("\n=== Page[{}] = stackable={} size={} items={} ===",
                page_idx, page.is_stackable, page.size, items.len());

            for (i, pi) in items.iter().enumerate() {
                let it = &pi.item;
                let q_str = format!("{:?}", it.quality);
                let code_clean = it.code.len() == 3 && !it.code.contains(' ');
                eprintln!("  [{:2}] offset={:>5}b len={:>4}b x={:2} y={:2} code={:6} q={:10} amount={:3} sl={} {}",
                    i, pi.raw_bit_offset, pi.raw_bit_length,
                    it.x, it.y, it.code, q_str, it.amount, it.stat_lists.len(),
                    if code_clean { "✅" } else { "⚠️" });
                if !it.stat_lists.is_empty() {
                    for (sl_idx, sl) in it.stat_lists.iter().enumerate() {
                        eprintln!("         sl[{}]: {} stats", sl_idx, sl.stats.len());
                        for (s_idx, s) in sl.stats.iter().enumerate() {
                            eprintln!("           [{:2}] id={:>4} param={:>4} value={}",
                                s_idx, s.id, s.param, s.value);
                        }
                    }
                }
            }
        }
    }
}
