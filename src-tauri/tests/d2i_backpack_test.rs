use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::protocol::d2s::parser::marker_offsets;

#[test]
fn test_d2i_backpack_items() {
    let d = r"D:\work_space\personal_workspace\d2r\开心图书馆长.d2s";
    let data = match std::fs::read(d) { Ok(d) => d, Err(_) => { eprintln!("SKIP"); return; } };
    let items = d2r_marketplace_lib::protocol::d2s::items::read_standard_items(&data).unwrap_or_default();

    println!("=== D2I Parser: {} items (bitstream order) ===", items.len());
    let mut sorted: Vec<&d2r_marketplace_lib::protocol::d2i::parser::ParsedItem> = items.iter().collect();
    sorted.sort_by_key(|pi| pi.raw_bit_offset);
    for (i, pi) in sorted.iter().enumerate() {
        let st = pi.item.stat_lists.iter().map(|sl| sl.stats.len()).sum::<usize>();
        let end = pi.raw_bit_offset + pi.raw_bit_length;
        println!("  [{:2}] {:4} start={:5} end={:5} mode={:?} Q={} ilvl={} stats={} id={}",
            i, pi.item.code, pi.raw_bit_offset, end, pi.item.mode, pi.item.quality.as_u8(),
            pi.item.item_level, st, pi.item.flags.identified());
    }

    let m = marker_offsets(&data);
    let mut stf_found = false;
    if let Some(jm_off) = m.first_jm {
        let stf_bit = (jm_off + 4) * 8 + 598;
        let stf_byte = stf_bit / 8;
        let bit_off = stf_bit % 8;
        if stf_byte + 14 <= data.len() {
            let mut r = BitReader::new(&data[stf_byte..]);
            r.skip_bits(bit_off);
            r.skip_bits(20); let _ = r.read_bit(); r.skip_bits(6);
            let _ = r.read_bit(); r.skip_bits(9);
            let _ = r.read_bit(); r.skip_bits(4);
            let _ = r.read_bit(); r.skip_bits(15);
            let _ = r.read_u8(3); let _ = r.read_u8(4);
            let _ = r.read_u8(4); let _ = r.read_u8(3); r.skip_bits(1);
            let _ = r.read_u8(3);
            stf_found = r.read_u16(10) == 650;
        }
    }

    // aqv stat dump
    if let Some(aqv) = items.iter().find(|pi| pi.item.code == "aqv") {
        println!("\n=== aqv (Arrows) ===");
        println!("D2I parser: Q={}, {} stats, ilvl={}, mode={:?}",
            aqv.item.quality.as_u8(),
            aqv.item.stat_lists.iter().map(|sl| sl.stats.len()).sum::<usize>(),
            aqv.item.item_level, aqv.item.mode);
        println!("Expected: Normal(2), quantity + type");
        if let Some(list) = aqv.item.stat_lists.first() {
            for s in &list.stats {
                println!("  D2I: stat id={} param={} value={}", s.id, s.param, s.value);
            }
        }
    }

    // sbw probe
    if let Some(sbw) = items.iter().find(|pi| pi.item.code == "sbw") {
        println!("\n=== sbw (Short Bow) ===");
        println!("D2I parser: Q={}, {} stats, ilvl={}",
            sbw.item.quality.as_u8(),
            sbw.item.stat_lists.iter().map(|sl| sl.stats.len()).sum::<usize>(),
            sbw.item.item_level);
        println!("Expected: Magic(4), lightning dmg 1-3, ilvl=1");

        if let Some(list) = sbw.item.stat_lists.first() {
            for s in &list.stats {
                println!("  D2I: stat id={} param={} value={}", s.id, s.param, s.value);
            }
        }

        let jm_off = m.first_jm.unwrap();
        let sbw_start_file = (jm_off + 4) * 8 + sbw.raw_bit_offset;
        let sbw_byte = sbw_start_file / 8;
        let _sbw_bit = sbw_start_file % 8;

        // Dump hex of sbw area (400 bits = 50 bytes + 8 margin)
        let dump_start = sbw_byte;
        let dump_len = 400_usize.div_ceil(8) + 8;
        let slice = &data[dump_start..std::cmp::min(dump_start + dump_len, data.len())];
        println!("\n--- sbw raw hex (file byte {}, {} bytes) ---", dump_start, dump_len.min(data.len().saturating_sub(dump_start)));
        for chunk in slice.chunks(16) {
            print!("  {:04x}:", dump_start + (slice.len() - chunk.len()));
            for b in chunk { print!(" {:02x}", b); }
            println!();
        }

        // Brute-force scan: find stat_id=51 (0x33)
        let sbw_limit = sbw.raw_bit_offset + sbw.raw_bit_length;
        let jm_data_start = (jm_off + 4) * 8;
        println!("\n--- Brute-force stat 51 (0x33) scan in sbw [{}..{}] ---", sbw.raw_bit_offset, sbw_limit);
        for offset in 0..(sbw.raw_bit_length - 9) {
            let bitpos = jm_data_start + sbw.raw_bit_offset + offset;
            let byte_idx = bitpos / 8;
            let bit_idx = bitpos % 8;
            if byte_idx + 2 > data.len() { break; }
            let mut r = BitReader::new(&data[byte_idx..]);
            r.skip_bits(bit_idx);
            let val = r.read_u16(9);
            if val == 0x33 {
                // Found stat 51 candidate
                let mut rv = BitReader::new(&data[byte_idx..]);
                rv.skip_bits(bit_idx + 9);
                // Try value widths 6..16
                for vb in [6u8, 7, 8, 9, 10, 16] {
                    let mut r2 = BitReader::new(&data[byte_idx..]);
                    r2.skip_bits(bit_idx + 9 + vb as usize);
                    let next_id = if r2.offset() + 9 <= r2.len_bits() { r2.read_u16(9) } else { 0 };
                    print!("  JM offset {:4} value_width={:2} next={:3}",
                        sbw.raw_bit_offset + offset + 9 + vb as usize, vb, next_id);
                    if next_id == 0x1FF {
                        // Check actual value
                        let v = rv.read_u32(vb) as i64;
                        println!(" value={} ← 0x1FF! ✓", v);
                    } else {
                        println!();
                    }
                }
                // Also check if this is the correct position:
                // stat 51 value=3 should be followed by 0x1FF
                // With save_bits=10 (from ItemStatCost), total = 9+10 = 19, 0x1FF at +28
                let mut r3 = BitReader::new(&data[byte_idx..]);
                r3.skip_bits(bit_idx + 9 + 10); // skip id + value at 10 bits
                let after = r3.read_u16(9);
                let v10 = {
                    let mut rv = BitReader::new(&data[byte_idx..]);
                    rv.skip_bits(bit_idx + 9);
                    rv.read_u32(10) as i64
                };
                if after == 0x1FF {
                    println!("  ✅ JM offset {}: stat 51, 10-bit value={}, followed by 0x1FF! CORRECT!", offset, v10);
                }
            }
        }

        // Also scan for 0x1FF
        println!("\n--- 0x1FF positions in sbw ---");
        for offset in 0..(sbw.raw_bit_length - 9) {
            let bitpos = jm_data_start + sbw.raw_bit_offset + offset;
            let byte_idx = bitpos / 8;
            let bit_idx = bitpos % 8;
            if byte_idx + 2 > data.len() { break; }
            let mut r = BitReader::new(&data[byte_idx..]);
            r.skip_bits(bit_idx);
            if r.read_u16(9) == 0x1FF {
                println!("  0x1FF at JM offset {} (total from item start)", offset);
            }
        }
    }

    // Final results
    let mut found: std::collections::HashSet<&str> = items.iter().map(|pi| pi.item.code.as_str()).collect();
    if stf_found { found.insert("stf"); }
    println!("\n=== Results ===");
    for exp in &["hp1","mp2","tbk","sbw","box","aqv","lsh","ibk","stf"] {
        println!("  {} {}", if found.contains(exp) { "✅" } else { "❌" }, exp);
    }
    assert!(found.contains("stf"), "stf should be found");
}
