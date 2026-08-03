//! 物品位级布局 (item bit layout) 显示函数
//!
//! 把一个 item 的 bit-level 编码以"位范围/长度/值/字段/含义"5 列表格化
//! 输出。包含 Huffman 解码、quality-specific 分支、stat stream 解析、
//! runeword prop_bits 多流处理等。
//!
//! 这个模块是从 construct.rs 拆出来的(原 dump_item_bits 296 行),
//! 单独的 bit-reader helpers 也提到这里以减少跨文件耦合。

use d2r_marketplace_lib::core::bitio::BitReader;
use d2r_marketplace_lib::core::encoding::decode_huffman_string;
use d2r_marketplace_lib::data::python_stats;
use d2r_marketplace_lib::protocol::common::stat_list::STAT_LIST_TERMINATOR;

use crate::helpers;

// ═══════════════════════════════════════════════
// Bit reader helpers
// ═══════════════════════════════════════════════

/// 读取 n bits 并返回 u32。n>32 时返回前 32 bits 并 skip 剩余。
pub fn read_bits_as(r: &mut BitReader, n: usize) -> u32 {
    if n <= 8 {
        r.read_u8(n as u8) as u32
    } else if n <= 16 {
        r.read_u16(n as u8) as u32
    } else if n <= 32 {
        r.read_u32(n as u8)
    } else {
        // >32 bits: read first 32, skip rest (caller discards for large skips like RealmData 128b)
        let val = r.read_u32(32);
        r.skip_bits(n - 32);
        val
    }
}

/// 读取 32 bits (D2 物品 flags 字段宽度)。
pub fn read_u32_at(r: &mut BitReader) -> u32 {
    r.read_u32(32)
}

// ═══════════════════════════════════════════════
// Bit layout dump
// ═══════════════════════════════════════════════

/// 把 payload 在 bit_offset 处的物品 bit-level 结构以可读表格输出。
/// 返回 String 给 CLI 直接打印。
pub fn dump_item_bits(payload: &[u8], bit_offset: usize) -> String {
    let mut r = BitReader::new(payload);
    r.seek(bit_offset);
    let start = r.offset();

    let mut lines = Vec::new();
    lines.push(format!("bit layout at offset {}b (byte {}):", bit_offset, bit_offset / 8));
    lines.push(format!("{:>12} {:>5} {:>25} {:30}  {}", "位范围", "长度", "值", "字段", "含义"));
    lines.push("=".repeat(110));

    let mut pos = start;

    let fl = read_u32_at(&mut r);
    let fl_end = r.offset();
    let ext = read_bits_as(&mut r, 3) as u8;
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, fl_end.wrapping_sub(1), 32, format!("0x{:08X}", fl), "Flags", "标记位域"));
    let ver_start = fl_end;
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", ver_start, r.offset().wrapping_sub(1), 3, ext, "Version(ext)", "D2R物品版本(5=标准)"));

    pos = r.offset(); let mode = read_bits_as(&mut r, 3); let mode_s = match mode {0=>"存放",1=>"装备",2=>"腰带",4=>"缓存",6=>"镶孔",_=>"?"};
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, format!("{}({})", mode, mode_s), "mode", ""));

    pos = r.offset(); let eq = read_bits_as(&mut r, 4);
    let eq_n = ["无","头","颈","身","右手","左手","右戒","左戒","腰","脚","手","副武右","副武左"];
    let eq_label = eq_n.get(eq as usize).unwrap_or(&"?");
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 4, format!("{}({})", eq, eq_label), "equip_loc", ""));

    pos = r.offset(); let px = read_bits_as(&mut r, 4);
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 4, px, "px", ""));
    pos = r.offset(); let py = read_bits_as(&mut r, 4);
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 4, py, "py", ""));
    pos = r.offset(); let pg = read_bits_as(&mut r, 3);
    let pg_n = ["装备","背包","装备内","交易","盒子","储藏箱","腰带","页7"];
    let pg_label = pg_n.get(pg as usize).unwrap_or(&"?");
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, format!("{}({})", pg, pg_label), "page", ""));

    if ext != 5 {
        pos = r.offset(); read_bits_as(&mut r, 8);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 8, "(pad)", "D2R_padding", "ext=5时的填充"));
    }

    let is_ear = fl & (1 << 16) != 0;
    let is_cp = fl & (1 << 21) != 0;

    if is_ear {
        pos = r.offset(); let ec = read_bits_as(&mut r, 3);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, ec, "ear_FileIndex", "职业"));
        pos = r.offset(); let el = read_bits_as(&mut r, 7);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 7, el, "ear_Level", "等级"));
        let mut pname = Vec::new();
        while r.remaining_bits() >= 7 {
            let c = read_bits_as(&mut r, 7) as u8;
            if c == 0 { break; }
            pname.push(c);
        }
        if !pname.is_empty() {
            pos = r.offset() - pname.len() * 7;
            let name_str = String::from_utf8_lossy(&pname);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), pname.len()*7, format!("{:?}", name_str), "ear_PlayerName", ""));
        }
        r.align_to_byte();
        lines.push(format!("  {:>4}  {:>5}  {:>25}  {:30} {}", r.offset().wrapping_sub(1), "", "", "--- ear end ---", ""));
        lines.push(format!("\n总计: {}b", r.offset() - start));
        return lines.join("\n");
    }

    // Huffman code
    let code = decode_huffman_string(&mut r);
    lines.push(format!("  {:>20}  {:>25}  {:30} {}", "", "", format!("{:?}", code), "code (Huffman)"));

    if is_cp {
        lines.push(format!("  {:>20}  {:>25}  {:30} {}", "", "", "", "--- compact end ---"));
        lines.push(format!("\n总计: {}b", r.offset() - start));
        return lines.join("\n");
    }

    pos = r.offset(); let sk = read_bits_as(&mut r, 3);
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, sk, "NumberOfSocketedItems", ""));
    pos = r.offset(); let uid = read_u32_at(&mut r);
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 32, format!("0x{:08X}", uid), "Id", "物品唯一ID"));
    pos = r.offset(); let ilvl = read_bits_as(&mut r, 7);
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 7, ilvl, "ItemLevel", ""));
    pos = r.offset(); let qb = read_bits_as(&mut r, 4);
    let qn_labels = ["None","Low","Normal","Superior","Magic","Set","Rare","Unique","Crafted"];
    let qn_label = qn_labels.get(qb as usize).unwrap_or(&"?");
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 4, format!("{}({})", qb, qn_label), "Quality", ""));

    pos = r.offset(); let hg = r.read_bit() != 0;
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if hg { "1" } else { "0" }, "HasMultipleGraphics", ""));
    if hg {
        pos = r.offset(); let gfx = read_bits_as(&mut r, 3);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, gfx, "  GraphicId", ""));
    }
    pos = r.offset(); let aa = r.read_bit() != 0;
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if aa { "1" } else { "0" }, "IsAutoAffix", ""));
    if aa {
        pos = r.offset(); let aaid = read_bits_as(&mut r, 11);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 11, aaid, "  AutoAffixId", ""));
    }

    // Quality branch
    match qb {
        1 | 3 => {
            pos = r.offset(); let fi = read_bits_as(&mut r, 3);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 3, fi, "  FileIndex(LOW/SUP)", ""));
        }
        4 => {
            pos = r.offset(); let mpre = read_bits_as(&mut r, 11);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 11, format!("0x{:03X}", mpre), "MagicPrefixId", ""));
            pos = r.offset(); let msuf = read_bits_as(&mut r, 11);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 11, format!("0x{:03X}", msuf), "MagicSuffixId", ""));
        }
        5 => {
            pos = r.offset(); let fi = read_bits_as(&mut r, 12);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 12, fi, "  FileIndex(SET)", ""));
        }
        7 => {
            pos = r.offset(); let fi = read_bits_as(&mut r, 12);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 12, fi, "  FileIndex(UNIQUE)", ""));
        }
        6 | 8 => {
            pos = r.offset(); let rp = read_bits_as(&mut r, 8);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 8, format!("0x{:02X}", rp), "RarePrefixId", ""));
            pos = r.offset(); let rs = read_bits_as(&mut r, 8);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 8, format!("0x{:02X}", rs), "RareSuffixId", ""));
            for i in 0..3 {
                pos = r.offset(); let hp = r.read_bit() != 0;
                lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if hp { "1" } else { "0" }, format!(" MagPre[{}]?", i), ""));
                if hp {
                    let p = r.offset();
                    let _id = read_bits_as(&mut r, 11);
                    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", p, r.offset().wrapping_sub(1), 11, _id, format!(" MagPre[{}]Id", i), ""));
                }
                pos = r.offset(); let hs = r.read_bit() != 0;
                lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if hs { "1" } else { "0" }, format!(" MagSuf[{}]?", i), ""));
                if hs {
                    let p = r.offset();
                    let _id = read_bits_as(&mut r, 11);
                    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", p, r.offset().wrapping_sub(1), 11, _id, format!(" MagSuf[{}]Id", i), ""));
                }
            }
        }
        _ => {}
    }

    let mut prop_list_idx: u32 = 0;
    if fl & (1 << 26) != 0 {
        pos = r.offset(); let rwid = read_bits_as(&mut r, 12);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 12, format!("0x{:03X}", rwid), "RunewordId", ""));
        let pl_pos = r.offset();
        prop_list_idx = read_bits_as(&mut r, 4);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pl_pos, r.offset().wrapping_sub(1), 4, prop_list_idx, "propListIdx", ""));
    }

    // Realm data
    pos = r.offset(); let hr = r.read_bit() != 0;
    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if hr { "1" } else { "0" }, "HasRealmData", ""));
    if hr {
        pos = r.offset(); read_bits_as(&mut r, 128);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 128, "(跳过)", "RealmData(128b)", "D2R/mod数据"));
    }

    // Type-dependent fields
    let is_armor_flag = helpers::is_armor(&code);
    let is_weapon_flag = helpers::is_weapon(&code);
    if is_armor_flag {
        let sb = python_stats::stat_bits(31) as usize;
        let sa = python_stats::stat_save_add(31) as u32;
        if sb > 0 {
            pos = r.offset();
            let arm = read_bits_as(&mut r, sb).wrapping_sub(sa);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), sb, arm, format!("Defense({}b)", sb), ""));
        }
    }
    if is_armor_flag || is_weapon_flag {
        let sb_max = python_stats::stat_bits(73) as usize;
        if sb_max > 0 {
            pos = r.offset();
            let md = read_bits_as(&mut r, sb_max).wrapping_sub(python_stats::stat_save_add(73) as u32);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), sb_max, md, format!("MaxDura({}b)", sb_max), ""));
            if md > 0 {
                let sb_cur = python_stats::stat_bits(72) as usize;
                if sb_cur > 0 {
                    pos = r.offset();
                    let cd = read_bits_as(&mut r, sb_cur).wrapping_sub(python_stats::stat_save_add(72) as u32);
                    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), sb_cur, cd, "CurDura", ""));
                }
                pos = r.offset(); let eb = read_bits_as(&mut r, 1);
                lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, eb, "Dura?bit", ""));
            }
        }
    }

    // Quantity (misc)
    if !is_weapon_flag && !is_armor_flag {
        pos = r.offset(); let hq = r.read_bit() != 0;
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 1, if hq { "1" } else { "0" }, "HasQuantity", ""));
        if hq {
            pos = r.offset(); let qv = read_bits_as(&mut r, 9);
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 9, qv, "Quantity", ""));
        }
    }

    // Sockets bit
    if fl & 0x800 != 0 {
        pos = r.offset(); let ms = read_bits_as(&mut r, 4);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 4, ms, "TotalSockets", ""));
    }
    if qb == 5 {
        pos = r.offset(); let sm = read_bits_as(&mut r, 5);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 5, format!("0x{:X}", sm), "SetItemMask", ""));
    }
    // lsh (灵石) special: 25 bits before stat stream
    if code == "lsh" {
        pos = r.offset();
        let _ls25 = read_bits_as(&mut r, 25);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", pos, r.offset().wrapping_sub(1), 25, format!("0x{:07X}", _ls25), "lsh special(25b)", "灵石, stat流整体跳过"));
        let total = r.offset() - start;
        lines.push(format!("\n总计: {}b ({}B+{}b)", total, total/8, total%8));
        return lines.join("\n");
    }
    // Stat stream
    lines.push(format!("  {:>20}  {:>25}  {:30} {}", "", "", "--- stat stream ---", ""));
    let mut stat_idx = 0u32;
    while r.remaining_bits() >= 9 {
        let sid_pos = r.offset();
        let sid = read_bits_as(&mut r, 9) as u16;
        if sid == STAT_LIST_TERMINATOR {
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", sid_pos, r.offset().wrapping_sub(1), 9, "0x1FF", "  TR terminator", "stat流终止"));
            break;
        }
        let nbits = python_stats::stat_bits(sid) as usize;
        let pbits = python_stats::stat_param_bits(sid) as usize;
        let _param = if pbits > 0 && r.remaining_bits() >= pbits { read_bits_as(&mut r, pbits) } else { 0 };
        if r.remaining_bits() < nbits {
            lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", sid_pos, r.offset().wrapping_sub(1), 9, sid, format!("  stat[{}] sid={}", stat_idx, sid), "UNDERRUN"));
            break;
        }
        let raw_val = read_bits_as(&mut r, nbits);
        let disp = (raw_val as i64).wrapping_sub(python_stats::stat_save_add(sid) as i64);
        let _pstr = if _param != 0 { format!(" param={}", _param) } else { String::new() };
        let sname = helpers::stat_label(sid).unwrap_or(&format!("s{}", sid)).to_string();
        let disp_s = format!("{}={}", sname, disp);
        let idx_s = format!("  stat[{}]", stat_idx);
        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} raw={raw_val} {nbits}b{_pstr}", sid_pos, r.offset().wrapping_sub(1), 9+nbits, disp_s, idx_s));
        stat_idx += 1;
        if stat_idx > 20 {
            lines.push(format!("  {:>4}  {:>25}  {:30} ({stat_idx} stats shown)", "", "", "  ..."));
            break;
        }
    } // end while
    // Additional stat streams from prop_bits (runeword/set bonus)
    if fl & (1 << 26) != 0 {
        let prop_bits = 1u32 << (prop_list_idx + 1);
        for shift in 0..7 {
            if prop_bits & (1 << shift) != 0 {
                lines.push(format!("  {:>20}  {:>25}  {:30} {}", "", "", "--- prop_bits stat stream ---", ""));
                loop {
                    if r.remaining_bits() < 9 { break; }
                    let sid_pos = r.offset();
                    let sid = read_bits_as(&mut r, 9) as u16;
                    if sid == STAT_LIST_TERMINATOR {
                        lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} {}", sid_pos, r.offset().wrapping_sub(1), 9, "0x1FF", "  TR terminator", "stat流终止"));
                        break;
                    }
                    let nbits = python_stats::stat_bits(sid) as usize;
                    let pbits = python_stats::stat_param_bits(sid) as usize;
                    let _param = if pbits > 0 && r.remaining_bits() >= pbits { read_bits_as(&mut r, pbits) } else { 0 };
                    if r.remaining_bits() < nbits { break; }
                    let raw_val = read_bits_as(&mut r, nbits);
                    let disp = (raw_val as i64).wrapping_sub(python_stats::stat_save_add(sid) as i64);
                    let _pstr = if _param != 0 { format!(" param={}", _param) } else { String::new() };
                    let sname = helpers::stat_label(sid).unwrap_or(&format!("s{}", sid)).to_string();
                    let disp_s = format!("{}={}", sname, disp);
                    let idx_s = format!("  prop_stat[{}]", stat_idx);
                    lines.push(format!("  {:>4}-{:>4}  {:3}b  {:>25}  {:30} raw={raw_val} {nbits}b{_pstr}", sid_pos, r.offset().wrapping_sub(1), 9+nbits, disp_s, idx_s));
                    if stat_idx > 20 {
                            lines.push(format!("  {:>4}  {:>25}  {:30} ({stat_idx} prop stats shown)", "", "", "  ..."));
                        break;
                    }
                }
            }
        }
    }

    let total = r.offset() - start;
    let shown_suffix = if stat_idx > 20 { " (仅显示前21stat)" } else { "" };
    lines.push(format!("\n总计: {}b ({}B+{}b){}", total, total/8, total%8, shown_suffix));
    lines.join("\n")
}
