//! D2S items 段解析（bit-level JM 编码，Python jm_parser.py 的 Rust 等价实现）。
//!
//! D2S 标准 halbu v105 layout:
//! - header (magic "D2S\0" + version + UTF-16 name + class/status + last_played)
//! - attributes (61 bytes bit-packed @ 0x341)
//! - skills (4 bytes bitmask)
//! - items (4 bytes u32 count + Item[])
//! - corpse items (4 bytes count + Item[])
//! - waypoints / quests / player_stats / menu / ...
//!
//! 标准 layout 的 Item[] 是 bit-level JM 编码,与 d2i items 段格式相同。
//! 魔改 layout (d2emu 仙道 mod) 走 `items_modified::read_items_with_quality`。

use crate::core::bitio::BitReader;
use crate::core::encoding::decode_huffman_string;
use crate::core::ParseResult;
use crate::protocol::common::{
    Item, ItemFlags, ItemLocation, ItemMode, ItemQuality, ItemStat,
};
use crate::protocol::common::stat_list::{StatList, STAT_LIST_TERMINATOR};
use crate::protocol::common::stat_table::StatTable;
use crate::protocol::d2i::parser::ParsedItem;
use crate::protocol::d2i::jm_reader::ParsedItemBuilder;
use super::items_modified::{detect_modified_layout, read_items_with_quality, ModifiedItem};
use super::parser::marker_offsets;

pub const SKILLS_SECTION_SIZE: usize = 32;
pub const SKILLS_COUNT: usize = 30;

/// Python ear validation (simplified): trust parsed ear items
/// D2R RealmData: if next 32-bit chunk has bit 16 set, consume 128 bits.
/// Matches Python `_scan_item` post-processing after each item.
fn consume_realm_data(reader: &mut BitReader, data: &[u8]) {
    if reader.remaining_bits() >= 32 {
        let mut peek_r = BitReader::new(data);
        peek_r.seek(reader.offset());
        let peek_val = peek_r.read_u32(32);
        if peek_val & (1 << 16) != 0 && reader.remaining_bits() >= 128 {
            reader.skip_bits(128);
        }
    }
}
// Huffman code + version alignment + full quality/stats parsing.

pub fn read_standard_items(data: &[u8]) -> ParseResult<Vec<ParsedItem>> {
    let m = marker_offsets(data);
    let jm_offset = match m.first_jm { Some(o) => o, None => return Ok(Vec::new()) };
    let search_end = m.jf.or(m.kf).unwrap_or(data.len()).min(data.len());
    if jm_offset + 4 > search_end { return Ok(Vec::new()); }
    let jm_data = &data[jm_offset..search_end];
    let payload = &jm_data[4..]; // skip JM(2B) + count(2B)
    let max_bit = payload.len() * 8;

    // 构建已知物品 code 集合（编译时）
    let all_codes: std::collections::HashSet<&str> =
        crate::protocol::d2i::legacy::game_items::ALL_ITEMS
            .iter()
            .map(|(code, _, _, _, _)| *code)
            .collect();
    let mut items: Vec<ParsedItem> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    // Sequential walk (matches Python jm_parser)
    // Phase 2 removed: Python doesn't have one, and it only creates false positives
    let mut bit_pos = 0usize;
    while bit_pos + 80 <= max_bit {
        let start = bit_pos;
        let mut reader = BitReader::new(payload);
        reader.seek(start);
        match try_parse_one(&mut reader, payload, jm_data, jm_offset) {
            Ok(Some(pi)) => {
                if pi.item.flags.raw > 0xE0000000 { bit_pos = reader.offset(); continue; }
                // Match Python: reject items with codes not in the known item database
                if !all_codes.contains(pi.item.code.as_str())
                    || pi.item.version_raw != 5
                { bit_pos = reader.offset(); continue; }
                let key = (pi.item.code.clone(), pi.raw_bit_offset);
                if seen.insert(key) {
                    items.push(pi);
                }
                bit_pos = reader.offset();
            }
            Ok(None) => { bit_pos += 1; }
            Err(_) => { bit_pos += 1; }
        }
    }
    // Sort by offset only (dedup against close-offset items is handled by seen set)
    items.sort_by_key(|a| a.raw_bit_offset);
    Ok(items)
}

/// 佣兵物品解析 — 第二个 JM block (merc_jm)。
pub fn read_merc_items(data: &[u8]) -> Vec<ParsedItem> {
    let m = marker_offsets(data);
    let jm_offset = match m.merc_jm { Some(o) => o, None => return Vec::new() };
    let search_end = m.jf.or(m.kf).unwrap_or(data.len()).min(data.len());
    if jm_offset + 4 > search_end { return Vec::new(); }
    let payload = &data[(jm_offset + 4)..];
    let max_bit = payload.len() * 8;

    let all_codes: std::collections::HashSet<&str> =
        crate::protocol::d2i::legacy::game_items::ALL_ITEMS
            .iter()
            .map(|(code, _, _, _, _)| *code)
            .collect();
    let mut items: Vec<ParsedItem> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Sequential walk
    let mut bit_pos = 0usize;
    while bit_pos + 80 <= max_bit {
        let start = bit_pos;
        let mut reader = BitReader::new(payload);
        reader.seek(start);
        match try_parse_one(&mut reader, payload, &[], jm_offset) {
            Ok(Some(pi)) => {
                if pi.item.flags.raw > 0xE0000000 { bit_pos = reader.offset(); continue; }
                if pi.item.version_raw != 5 || pi.item.code.len() != 3
                    || !pi.item.code.chars().all(|c| c.is_ascii_alphanumeric())
                { bit_pos = reader.offset(); continue; }
                let key = (pi.item.code.clone(), pi.raw_bit_offset);
                if seen.insert(key) { items.push(pi); }
                bit_pos = reader.offset();
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
    // Scan supplement
    for start in (0..max_bit.saturating_sub(100)).step_by(8) {
        if seen.iter().any(|(_, off): &(String, usize)| *off == start) { continue; }
        let mut check_r = BitReader::new(payload);
        check_r.seek(start);
        if check_r.remaining_bits() < 80 { break; }
        let flags_raw = check_r.read_u32(32);
        let ver = check_r.read_u8(3);
        if ver < 3 && (flags_raw >> 16) & 1 == 0 { continue; }
        let _loc = check_r.read_u8(3);
        let _slot = check_r.read_u8(4);
        let _px = check_r.read_u8(4);
        let _py = check_r.read_u8(4);
        let _pg = check_r.read_u8(3);
        let code = decode_huffman_string(&mut check_r).trim().to_string();
        if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphanumeric()) || !all_codes.contains(code.as_str()) {
            continue;
        }
        let key = (code.clone(), start);
        if !seen.insert(key) { continue; }
        let mut reader = BitReader::new(payload);
        reader.seek(start);
        if let Ok(Some(pi)) = try_parse_one(&mut reader, payload, &[], jm_offset)
            && pi.item.flags.raw <= 0xE0000000 && pi.item.version_raw == 5
                && pi.item.code.len() == 3
                && pi.item.code.chars().all(|c| c.is_ascii_alphanumeric())
            { items.push(pi); }
    }
    // Dedup: same code within 64 bits = same item
    items.sort_by_key(|a| a.raw_bit_offset);
    let mut deduped: Vec<ParsedItem> = Vec::with_capacity(items.len());
    for pi in items {
        let too_close = deduped.iter().any(|ex| {
            ex.item.code == pi.item.code
            && pi.raw_bit_offset.abs_diff(ex.raw_bit_offset) < 64
        });
        if !too_close { deduped.push(pi); }
    }
    deduped
}
#[allow(unused_assignments)]
fn try_parse_one(
    reader: &mut BitReader, jm_payload: &[u8], jm_data: &[u8], _jm_offset: usize,
) -> ParseResult<Option<ParsedItem>> {
    let start = reader.offset();
    if reader.remaining_bits() < 80 { return Ok(None); }

    let mut flags_raw = reader.read_u32(32);
    let mut ver = reader.read_u8(3);
    let mut m = reader.read_u8(3);
    let mut loc = reader.read_u8(4);
    let mut px = reader.read_u8(4);
    let mut py = reader.read_u8(4);
    let mut pg = reader.read_u8(3);
    let code = decode_huffman_string(reader);
    let mut code = code.trim().to_string();

    // Version alignment: if ext != 5 and !ear, skip forward and re-read
    if ver != 5 && (flags_raw >> 16) & 1 == 0 {
        // Python: scan for correct byte-aligned offset, then recursive _scan_item(data, _new_bo)
        let mut new_start = start + 8; // default: skip 1 byte forward
        for skip in 1..33 {
            let new_pos = start + skip * 8;
            if new_pos + 35 > jm_payload.len() * 8 { break; }
            let mut peek_r = BitReader::new(jm_payload);
            peek_r.seek(new_pos);
            let pf = peek_r.read_u32(32);
            let pv = peek_r.read_u8(3);
            if pv == 5 && pf.count_ones() <= 4 {
                new_start = new_pos;
                break;
            }
        }
        // seek to the found alignment (or default start+8 if none found)
        reader.seek(new_start);
        flags_raw = reader.read_u32(32);
        ver = reader.read_u8(3);
        m = reader.read_u8(3);
        loc = reader.read_u8(4);
        px = reader.read_u8(4);
        py = reader.read_u8(4);
        pg = reader.read_u8(3);
        code = decode_huffman_string(reader);
        code = code.trim().to_string();
    }

    // Python: code validation — if not in game database, scan forward
    if (flags_raw >> 16) & 1 == 0 {
        let code_valid = code.len() == 3
            && code.chars().all(|c| c.is_ascii_alphanumeric())
            && code.chars().any(|c| c.is_ascii_alphabetic());
        if !code_valid {
            let max_scan = std::cmp::min(32usize, reader.remaining_bits().div_ceil(8));
            for skip in 1..=max_scan {
                let new_pos = start + skip * 8;
                if new_pos + 35 > jm_payload.len() * 8 { break; }
                let mut peek_r = BitReader::new(jm_payload);
                peek_r.seek(new_pos);
                let pf = peek_r.read_u32(32);
                let pv = peek_r.read_u8(3);
                if pv == 5 && pf.count_ones() <= 4 {
                    // Check if the code at that position is also valid
                    let mut code_r = BitReader::new(jm_payload);
                    code_r.seek(new_pos + 53);
                    let candidate = decode_huffman_string(&mut code_r).trim().to_string();
                    let cv = candidate.len() == 3
                        && candidate.chars().all(|c| c.is_ascii_alphanumeric())
                        && candidate.chars().any(|c| c.is_ascii_alphabetic());
                    if cv {
                        reader.seek(new_pos);
                        flags_raw = reader.read_u32(32);
                        ver = reader.read_u8(3);
                        m = reader.read_u8(3);
                        loc = reader.read_u8(4);
                        px = reader.read_u8(4);
                        py = reader.read_u8(4);
                        pg = reader.read_u8(3);
                        code = decode_huffman_string(reader);
                        code = code.trim().to_string();
                        break;
                    }
                }
            }
        }
    }

    // Python: ear check FIRST (before compact)
    if (flags_raw >> 16) & 1 == 1 {
        ver = 5; // ear bit set → version alignment skipped; force ver=5
        reader.align_to_byte();
        consume_realm_data(reader, jm_payload);
        return Ok(Some(ParsedItemBuilder::new()
            .flags(flags_raw).version(ver).position(m, loc, px, py, pg)
            .quality(ItemQuality::None)
            .amount(0)
            .raw_bit_range(start, reader.offset() - start)
            .build()));
    }
    if (flags_raw >> 21) & 1 == 1 {
        // Python: all compact items in _REALM_COMPACT → no qf/qty reading
        // Python: if aligned, r.p += 8; else align_to_byte()
        if reader.offset().is_multiple_of(8) { reader.skip_bits(8); }
        else { reader.align_to_byte(); }
        // Python: post-parse alignment recovery for specific codes
        if code == "gcv" || code == "vps" || code == "wms" {
            let max_scan = std::cmp::min(64usize, reader.remaining_bits() / 8);
            for _ in 0..max_scan {
                if reader.remaining_bits() < 40 { break; }
                let mut peek_r = BitReader::new(jm_payload);
                peek_r.seek(reader.offset());
                let pf = peek_r.read_u32(32);
                let pv = peek_r.read_u8(3);
                if pv == 5 && pf.count_ones() <= 4 {
                    break; // found next valid item
                }
                reader.skip_bits(8);
            }
        }
        consume_realm_data(reader, jm_payload);
        return Ok(Some(ParsedItemBuilder::new()
            .flags(flags_raw).version(ver).position(m, loc, px, py, pg).code(code)
            .quality(ItemQuality::Normal)
            .amount(0)
            .raw_bit_range(start, reader.offset() - start)
            .build()));
    }
    // Python: code may be invalid; parse as misc, filter later in caller

    let _soc_3 = reader.read_u8(3);
    let uid = reader.read_u32(32);
    let ilvl = reader.read_u8(7);
    let qb = reader.read_u8(4);
    if reader.remaining_bits() < 12 { return Ok(None); }
    if reader.read_bit() == 1 { reader.skip_bits(3); }
    if reader.read_bit() == 1 { reader.skip_bits(11); }

    let mut magic_prefix = None;
    let mut magic_suffix = None;
    let (quality, parsed_uid, parsed_sid) = match qb {
        7 => (ItemQuality::Unique, Some(reader.read_u16(12)), None),
        5 => (ItemQuality::Set, None, Some(reader.read_u16(12))),
        4 => {
            let p = reader.read_u16(11);
            let s = reader.read_u16(11);
            if p > 0 { magic_prefix = Some(p); }
            if s > 0 { magic_suffix = Some(s); }
            (ItemQuality::Magic, None, None)
        }
        6 => { reader.read_u16(16); for _ in 0..6 { if reader.read_bit() == 1 { reader.skip_bits(11); } }
              (ItemQuality::Rare, None, None) }
        8 => { reader.read_u16(16); for _ in 0..6 { if reader.read_bit() == 1 { reader.skip_bits(11); } }
              (ItemQuality::Crafted, None, None) }
        1 | 3 => { reader.skip_bits(3); (if qb == 1 { ItemQuality::Low } else { ItemQuality::Superior }, None, None) }
        _ => (ItemQuality::Normal, None, None)
    };

    let mut prop_bits: u32 = 0;
    let mut num_sockets: u8 = 0;
    if (flags_raw >> 26) & 1 == 1 { reader.read_u16(12); prop_bits = 1u32 << (reader.read_u8(4) + 1); }
    if code == "tbk" || code == "ibk" { reader.skip_bits(5); }
    if reader.read_bit() == 1 { reader.skip_bits(128); }

    // Type-dependent fields (Python order: a → m → w)
    let is_armor = crate::data::items_base::is_armor(&code);
    let is_weapon = crate::data::items_base::is_weapon(&code);
    // Type-dependent fields — read durability from bitstream
    let (cur_dur, max_dur, def_value) = if is_armor {
        read_type_fields(reader, "a")
    } else if is_weapon {
        read_type_fields(reader, "w")
    } else {
        read_type_fields(reader, "m")
    };
    if code == "lsh" { reader.skip_bits(25); reader.align_to_byte(); consume_realm_data(reader, jm_payload);
        let bit_len = reader.offset() - start;
        return Ok(Some(ParsedItemBuilder::new()
            .flags(flags_raw).version(ver).position(m, loc, px, py, pg).code(code)
            .quality(ItemQuality::Normal)
            .amount(0)
            .durability(cur_dur, max_dur)
            .defense(def_value)
            .raw_bit_range(start, bit_len)
            .build()));
    }
    if (flags_raw >> 11) & 1 == 1 { num_sockets = reader.read_u8(4); }
    // Python: set bonus bits AND into prop_bits before stat streams
    if qb == 5 { prop_bits |= reader.read_u8(5) as u32; }
    let (stat_lists, stat_clean) = parse_stat_lists(reader, jm_data, prop_bits, quality, &code);
    reader.align_to_byte();
    // Conditional forward scan: only when stat stream didn't terminate cleanly (matches Python _scan_complete_item)
    if !stat_clean && reader.remaining_bits() >= 40 {
        let save = reader.offset();
        let mut peek_r = BitReader::new(jm_payload);
        peek_r.seek(save);
        let pf = peek_r.read_u32(32);
        let pv = peek_r.read_u8(3);
        if pv != 5 || pf.count_ones() > 4 {
            let max_scan = std::cmp::min(64usize, reader.remaining_bits() / 8);
            for try_b in 1..=max_scan {
                let pos = save + try_b * 8;
                if pos + 35 > jm_payload.len() * 8 { break; }
                let mut fwd_r = BitReader::new(jm_payload);
                fwd_r.seek(pos);
                let ff = fwd_r.read_u32(32);
                let fv = fwd_r.read_u8(3);
                if fv == 5 && ff.count_ones() <= 4 {
                    reader.seek(pos);
                    break;
                }
            }
        }
    }
    let bit_len = reader.offset() - start;
    Ok(Some(ParsedItemBuilder::new()
        .flags(flags_raw).version(ver).position(m, loc, px, py, pg).code(code)
        .uid(uid).ilvl(ilvl).quality(quality)
        .stat_lists(stat_lists).num_sockets(num_sockets)
        .amount(0)
        .durability(cur_dur, max_dur)
        .defense(def_value)
        .magic_prefix(magic_prefix).magic_suffix(magic_suffix)
        .unique_id(parsed_uid).set_id(parsed_sid)
        .raw_bit_range(start, bit_len)
        .build()))
}

/// Skip type-dependent fields AND capture durability/defense values.
/// Returns (current_durability, max_durability, defense).
fn read_type_fields(reader: &mut BitReader, ty: &str) -> (u8, u8, u16) {
    match ty {
        "a" => {
            // Stat 31 = defense (11 bits, stored actual value)
            let bits = crate::data::python_stats::stat_bits(31);
            let def_value = if bits > 0 && reader.remaining_bits() >= bits {
                reader.read_u16(bits as u8) as u16
            } else { 0 };
            // Stat 73 = max durability (8 bits, stored doubled)
            let max_dur = if reader.remaining_bits() >= 8 {
                reader.read_u8(8)
            } else { 0 };
            // Stat 72 = current durability (9 bits, stored doubled)
            let cur_dur = if max_dur > 0 && reader.remaining_bits() >= 9 {
                reader.read_u16(9) as u8
            } else {
                if reader.remaining_bits() >= 1 { reader.read_bit(); }
                0
            };
            if reader.remaining_bits() >= 1 { reader.read_bit(); }
            (cur_dur, max_dur, def_value)
        }
        "w" => {
            let max_dur = if reader.remaining_bits() >= 8 {
                reader.read_u8(8)
            } else { 0 };
            let cur_dur = if max_dur > 0 && reader.remaining_bits() >= 9 {
                reader.read_u16(9) as u8
            } else {
                if reader.remaining_bits() >= 1 { reader.read_bit(); }
                0
            };
            if reader.remaining_bits() >= 1 { reader.read_bit(); }
            (cur_dur, max_dur, 0)
        }
        _ => {
            if reader.remaining_bits() >= 10 && reader.read_bit() == 1 {
                reader.skip_bits(9);
            }
            (0, 0, 0)
        }
    }
}


fn parse_stat_lists(
    reader: &mut BitReader, _jm_data: &[u8], prop_bits: u32,
    _quality: ItemQuality, _code: &str,
) -> (Vec<StatList>, bool) {
    let mut lists = Vec::new();
    let mut clean = true;
    let (sl, sc) = parse_stat_stream(reader);
    if let Some(sl) = sl { lists.push(sl); }
    clean = clean && sc;
    for shift in 0..7 {
        if prop_bits & (1 << shift) != 0 {
            let (sl, sc) = parse_stat_stream(reader);
            if let Some(sl) = sl { lists.push(sl); }
            clean = clean && sc;
        }
    }
    (lists, clean)
}

fn parse_stat_stream(reader: &mut BitReader) -> (Option<StatList>, bool) {
    let start = reader.offset();
    let mut stats = Vec::new();
    let mut terminated_with_0x1ff = false;

    // 获取 stat 表用于 encoding/descfunc 拆分
    let table = if crate::data::stat_loader::has_runtime_table() {
        crate::data::stat_loader::build_runtime_table()
    } else {
        crate::data::stat_cost::build_stat_table()
    };

    while reader.remaining_bits() >= 9 {
        let id = reader.read_u16(9);
        if id == STAT_LIST_TERMINATOR { terminated_with_0x1ff = true; break; }
        // Python: read param bits if any
        let pbits = crate::data::python_stats::stat_param_bits(id);
        let param_val = if pbits > 0 && reader.remaining_bits() >= pbits {
            reader.read_u32(pbits as u8)
        } else {
            0
        };
        // Python: read value bits
        let nbits = crate::data::python_stats::stat_bits(id);
        let raw_val = if nbits > 0 && reader.remaining_bits() >= nbits {
            reader.read_u32(nbits as u8) as i64
        } else if reader.remaining_bits() >= 9 {
            reader.read_u32(9) as i64
        } else {
            break;
        };
        // Python: apply save_add (subtract offset from raw value)
        let save_add = crate::data::python_stats::stat_save_add(id) as i64;
        let val = raw_val.wrapping_sub(save_add);

        // 查 stat table 确定 encoding/descfunc，填充 skill 拆分字段
        let prop = table.get(id);
        let (skill_tab, skill_level, skill_id, max_charges) = if prop.descfunc == 14 {
            // descfunc=14: item_addskill_tab — tab = param & 0x7, 等级 = value
            (Some((param_val & 0x7) as u8), Some(val.clamp(0, u16::MAX as i64) as u16), None, None)
        } else if prop.encoding == 1 {
            // encode=1: item_singleskill — param IS skill_id, value IS skill_level
            (None, Some(val as u16), Some(param_val as u16), None)
        } else if prop.encoding == 2 {
            // encode=2: item_skillon* — param split: low6=level, high10=skill_id
            (None, Some((param_val & 0x3f) as u16), Some(((param_val >> 6) & 0x3ff) as u16), None)
        } else if prop.encoding == 3 {
            // encode=3: item_charged_skill — value split into current+max charges
            let max = Some(((raw_val >> 8) & 0xff) as u8);
            (None, Some((param_val & 0x3f) as u16), Some(((param_val >> 6) & 0x3ff) as u16), max)
        } else {
            (None, None, None, None)
        };

        let final_val = if prop.encoding == 3 { (raw_val & 0xff) - save_add } else { val };
        stats.push(ItemStat {
            id, value: final_val, param: param_val,
            skill_tab, skill_level, skill_id, max_charges,
        });
        // Python: grouped siblings (NP>1)
        for mi in 1..crate::data::python_stats::stat_np(id) {
            let mnb = crate::data::python_stats::stat_bits(id + mi as u16);
            if reader.remaining_bits() >= mnb {
                let sv = reader.read_u32(mnb as u8) as i64;
                let sv_add = crate::data::python_stats::stat_save_add(id + mi as u16) as i64;
                let sv_val = sv.wrapping_sub(sv_add);
                stats.push(ItemStat { id: id + mi as u16, value: sv_val, param: 0,
                    skill_tab: None, skill_level: None, skill_id: None, max_charges: None });
            }
        }
    }
    if stats.is_empty() && reader.offset() == start {
        (None, terminated_with_0x1ff)
    } else {
        (Some(StatList { stats }), terminated_with_0x1ff)
    }
}

pub fn reparse_d2s_body(
    data: &[u8], jm_offset: usize, pi: &ParsedItem, table: &StatTable,
) -> Option<(u32, u8, ItemQuality, Vec<StatList>, u16, u16)> {
    let bit_pos = pi.raw_bit_offset;
    let file_off = jm_offset + bit_pos / 8;
    let bit_off = bit_pos % 8;
    if file_off + 36 > data.len() { return None; }
    let mut r = BitReader::new(&data[file_off..]);
    r.skip_bits(bit_off);
    let _flags = ItemFlags::read(&mut r).ok()?;
    r.skip_bits(3+3+4+4+4+3);
    decode_huffman_string(&mut r);
    if _flags.simple_item() { r.read_u8(1); } else { r.read_u8(3); }
    let uid = r.read_u32(32);
    let ilvl = r.read_u8(7);
    let qb = r.read_u8(4);
    let quality = match qb { 0 => ItemQuality::None, 1 => ItemQuality::Low, 2 => ItemQuality::Normal,
        3 => ItemQuality::Superior, 4 => ItemQuality::Magic, 5 => ItemQuality::Set,
        6 => ItemQuality::Rare, 7 => ItemQuality::Unique, 8 => ItemQuality::Crafted, v => ItemQuality::Unknown(v) };
    if r.read_u8(1) == 1 { r.skip_bits(3); }
    if r.read_u8(1) == 1 { r.skip_bits(11); }
    let mut unique_id: u16 = 0; let mut set_id: u16 = 0;
    match qb {
        1 | 3 => { r.skip_bits(3); }
        4 => { r.read_u16(11); r.read_u16(11); }
        5 => { set_id = r.read_u16(12); }
        7 => { unique_id = r.read_u16(12); }
        6 | 8 => { r.read_u16(16); for _ in 0..6 { if r.read_bit() == 1 { r.skip_bits(11); } } }
        _ => {}
    }
    let mut stat_lists = Vec::new();
    if let Some(stats) = read_stat_strict(&mut r, table) { stat_lists.push(StatList { stats }); }
    Some((uid, ilvl, quality, stat_lists, unique_id, set_id))
}

fn read_stat_strict(r: &mut BitReader, table: &StatTable) -> Option<Vec<ItemStat>> {
    let mut stats = Vec::new();
    loop {
        if r.remaining_bits() < 9 { return None; }
        let id = r.read_u16(9);
        if id == STAT_LIST_TERMINATOR { return Some(stats); }
        let prop = table.get(id);
        let s = ItemStat::read(r, id, &prop).ok()?;
        stats.push(s);
        let sub_count = crate::protocol::common::version_dispatch::FieldSet::sub_property_count(id);
        for offset in 1..=sub_count as u16 {
            if let Ok(s) = ItemStat::read(r, id + offset, &table.get(id + offset)) { stats.push(s); }
        }
    }
}

/// 读取技能段（"if" 段 30 字节原始等级数据）。
pub fn read_skills(data: &[u8]) -> Option<Vec<u8>> {
    let m = marker_offsets(data);
    let if_offset = m.if_?;
    if if_offset + SKILLS_SECTION_SIZE > data.len() || &data[if_offset..if_offset + 2] != b"if" {
        return None;
    }
    Some(data[if_offset + 2..if_offset + 2 + SKILLS_COUNT].to_vec())
}

/// 已装备物品 — 从标准解析结果中过滤 ItemMode::Equipped。
pub fn parse_equipped(data: &[u8]) -> Vec<ParsedItem> {
    let all = read_standard_items(data).unwrap_or_default();
    // Associate socketed sub-items before filtering for equipped
    let all = associate_socketed_items(&all);
    all.into_iter()
        .filter(|pi| pi.item.mode == ItemMode::Equipped)
        .collect()
}

/// 读取 d2s items 段。
///
/// 魔改 layout → `items_modified`，标准 D2S → 标准 JM 解析。
pub fn read_items(data: &[u8]) -> ParseResult<Vec<ParsedItem>> {
    if detect_modified_layout(data) {
        let mod_items = read_items_with_quality(data);
        return Ok(mod_items.into_iter().map(modified_to_parsed).collect());
    }
    read_standard_items(data)
}

fn modified_to_parsed(mi: ModifiedItem) -> ParsedItem {
    let quality = match mi.quality_byte {
        0 => ItemQuality::None,
        1 => ItemQuality::Low,
        2 => ItemQuality::Normal,
        3 => ItemQuality::Superior,
        4 => ItemQuality::Magic,
        5 => ItemQuality::Set,
        6 => ItemQuality::Rare,
        7 => ItemQuality::Unique,
        8 => ItemQuality::Crafted,
        v => ItemQuality::Unknown(v),
    };
    // raw_data[0..2] LE 主数值: armor/shield = defense; unique(7)/set(5) = id。
    // 仅当 quality 非 unique/set 时才把主数值当 defense。
    let defense = match mi.quality_byte {
        5 | 7 => 0u16,
        _ => u16::from_le_bytes([mi.raw_data[0], mi.raw_data[1]]),
    };
    ParsedItem {
        page_index: 0,
        item: Item {
            flags: Default::default(),
            version_raw: 105,
            mode: ItemMode::Stored,
            location: ItemLocation::None,
            x: 0,
            y: 0,
            page: None,
            code: mi.code.clone(),
            num_sockets: 0,
            id: 0,
            item_level: mi.i_lvl,
            quality,
            stat_lists: Vec::new(),
            amount: 1,
            socketed_items: Vec::new(),
            current_durability: 0,
            max_durability: 0,
            defense,
            unique_id: mi.unique_id,
            set_id: mi.set_id,
        },
        raw_bit_offset: 0,
        raw_bit_length: 0,
        is_socketed_subitem: false,
        is_pseudo_unverified: false,
        magic_prefix_id: None,
        magic_suffix_id: None,
    }
}

/// Post-process a flat list of ParsedItems to build parent-child socket relationships.
/// A socketed child is any item whose raw_bit_offset falls within a preceding
/// item's range `[start, start + bit_length)`. Standalone items are kept as-is.
pub fn associate_socketed_items(items: &[ParsedItem]) -> Vec<ParsedItem> {
    let mut sorted: Vec<ParsedItem> = items.to_vec();
    sorted.sort_by_key(|pi| pi.raw_bit_offset);
    let mut result: Vec<(ParsedItem, usize)> = Vec::new(); // (item, extended_end_offset)
    for pi in sorted {
        let ps = pi.raw_bit_offset;
        let pe = ps + pi.raw_bit_length;
        // 子物品必须是 Socket mode，且偏移在某个父物品的扩展范围内
        let parent_idx = result.iter().position(|(parent, ext_end)| {
            pi.item.mode == ItemMode::Socket
                && ps >= parent.raw_bit_offset && ps <= *ext_end
        });
        if let Some(idx) = parent_idx {
            let (parent, ext_end) = &mut result[idx];
            parent.item.socketed_items.push(pi.item.clone());
            // 扩展父物品的范围以覆盖这个子物品
            *ext_end = (*ext_end).max(pe);
        } else {
            result.push((pi, pe));
        }
    }
    result.into_iter().map(|(item, _)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_item0_kaixinxiedi() {
        let path = r"D:\work_space\personal_workspace\d2r\开心邪帝.d2s";
        if !std::path::Path::new(path).exists() { eprintln!("SKIP: 本地诊断文件缺失"); return; }
        let data = std::fs::read(path).expect("read file");
        let items = read_standard_items(&data).expect("parse items");
        assert!(items.len() >= 100, "expected >=100 items, got {}", items.len());
        assert_eq!(items[0].item.code, "cm3");
        assert_eq!(items[0].item.item_level, 16);
    }

    use crate::protocol::common::{ItemFlags, ItemLocation, ItemPage};

    fn make_test_item(code: &str, mode: ItemMode, offset: usize, length: usize, is_rw: bool) -> ParsedItem {
        let mut flags = ItemFlags { raw: 0 };
        if is_rw { flags.raw |= 1 << 26; }
        ParsedItem {
            page_index: 0,
            item: Item {
                flags,
                version_raw: 105,
                mode,
                location: ItemLocation::None, x: 0, y: 0,
                page: Some(ItemPage::Equipped),
                code: code.to_string(),
                num_sockets: 0, id: 0, item_level: 1,
                quality: ItemQuality::Normal,
                stat_lists: Vec::new(), amount: 1,
                socketed_items: Vec::new(),
                current_durability: 0, max_durability: 0,
                defense: 0,
                unique_id: None, set_id: None,
            },
            raw_bit_offset: offset, raw_bit_length: length,
            is_socketed_subitem: false, is_pseudo_unverified: false,
            magic_prefix_id: None, magic_suffix_id: None,
        }
    }

    #[test]
    fn test_associate_socketed_basic() {
        // hla(硬皮甲) 在偏移 0，长度 200 bits，后跟 r05(Tal) r07(Eth) 两个 Socket 子物品
        let items = vec![
            make_test_item("hla", ItemMode::Equipped, 0, 200, true),
            make_test_item("r05", ItemMode::Socket, 200, 80, false),
            make_test_item("r07", ItemMode::Socket, 280, 80, false),
        ];
        let result = associate_socketed_items(&items);
        assert_eq!(result.len(), 1, "3 items → 1 parent, 2 socketed");
        assert_eq!(result[0].item.code, "hla");
        assert_eq!(result[0].item.socketed_items.len(), 2);
        assert_eq!(result[0].item.socketed_items[0].code, "r05");
        assert_eq!(result[0].item.socketed_items[1].code, "r07");
    }

    #[test]
    fn test_associate_socketed_boundary_exact() {
        // 子物品偏移恰好等于父物品结束位置 — 之前 < pe 的 bug 会漏掉
        let items = vec![
            make_test_item("hla", ItemMode::Equipped, 100, 50, true),
            make_test_item("r05", ItemMode::Socket, 150, 30, false),
        ];
        let result = associate_socketed_items(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].item.socketed_items.len(), 1);
        assert_eq!(result[0].item.socketed_items[0].code, "r05");
    }

    #[test]
    fn test_associate_non_socket_not_absorbed() {
        // 非 Socket mode 的相邻物品不应被关联
        let items = vec![
            make_test_item("hla", ItemMode::Equipped, 0, 100, false),
            make_test_item("rin", ItemMode::Equipped, 100, 50, false),
        ];
        let result = associate_socketed_items(&items);
        assert_eq!(result.len(), 2);
        assert_eq!(result[1].item.socketed_items.len(), 0);
    }
}
