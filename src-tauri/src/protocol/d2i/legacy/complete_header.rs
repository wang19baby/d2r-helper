//! Complete header skipping logic for D2I legacy items.
//!
//! Extracted from `item.rs` during Phase 10 refactoring.
//! Contains `skip_non_simple_complete_header` and all supporting functions
//! for parsing the 1500-line complete item body (quality-specific fields,
//! magic properties, set bonuses, runeword bonuses, etc.).
//!
//! Callers should use `skip_non_simple_complete_header` to skip over the
//! complete header without extracting stat values — useful for two-pass parsing.

use super::bit_reader::BitReader;
use super::game_items::ALL_ITEMS;
use super::magical_props::MAGICAL_PROPS;
use crate::protocol::common::stat::ItemStat;
use crate::protocol::common::stat_list::StatList;
use crate::protocol::common::stat_table::StatTable;

/// Item quality enum matching the game's values
#[repr(u8)]
enum Quality {
    Low = 1,
    Normal = 2,
    Superior = 3,
    Magic = 4,
    Set = 5,
    Rare = 6,
    Unique = 7,
    Crafted = 8,
}

impl Quality {
    fn from_bits(v: u8) -> Self {
        match v {
            1 => Quality::Low,
            2 => Quality::Normal,
            3 => Quality::Superior,
            4 => Quality::Magic,
            5 => Quality::Set,
            6 => Quality::Rare,
            7 => Quality::Unique,
            8 => Quality::Crafted,
            _ => Quality::Normal,
        }
    }
}

/// Look up an item's categories from the game constants.
/// Returns (is_armor, is_weapon, is_shield)
pub fn lookup_item_category(code: &str) -> (bool, bool, bool) {
    let code = code.trim();
    for (c, _, is_a, is_w, is_s) in ALL_ITEMS.iter() {
        if *c == code {
            return (*is_a, *is_w, *is_s);
        }
    }
    (false, false, false)
}

/// Skip a non-simple item's complete header.
///
/// Reads past id / level / quality / quality-specific fields / durability /
/// sockets / runeword / personalized / v105 preamble / magic properties /
/// set bonuses / runeword bonuses.
///
/// Returns the item's quality value (1-8).
///
/// New code should call this then read stat_list(s) at the returned position.
/// Caller has already parsed compact header (flags / location / code / num_sockets).
pub fn skip_non_simple_complete_header(
    reader: &mut BitReader,
    item_type: &str,
    _identified: bool,
    socketed: bool,
    personalized: bool,
    given_runeword: bool,
    is_ear: bool,
    version: u8,
    force_sockets: bool,
    table: &StatTable,
) -> Result<u8, String> {
    skip_non_simple_item_body_inner(
        reader, item_type, _identified, socketed, personalized, given_runeword, is_ear, version, force_sockets, table,
    )
}

/// Read complete header fields up to (but not including) magic properties.
/// Returns (quality_bits, is_set_quality, plist_flag).
/// Used by both skip and parse code paths.
fn read_pre_magic_fields(
    reader: &mut BitReader,
    item_type: &str,
    socketed: bool,
    personalized: bool,
    given_runeword: bool,
    version: u8,
    force_sockets: bool,
    table: &StatTable,  // for stat-table-based bit widths (durability, defense)
) -> Result<(
    u8, u8, bool, u8, Option<u16>, Option<u16>, u8, u8
), String> {
    let (is_armor, is_weapon, is_shield) = lookup_item_category(item_type);

    let _item_id = reader.read_u32(32);
    let level = reader.read_u8(7);
    let quality_bits = reader.read_u8(4);
    let _quality = Quality::from_bits(quality_bits);

    // multi_pic
    if reader.read_bit() == 1 { let _ = reader.read_u8(3); }
    // class_specific
    if reader.read_bit() == 1 { let _ = reader.read_u16(11); }

    // quality-specific
    let unique_id: Option<u16>;
    let set_id: Option<u16>;
    match quality_bits {
        1 => { let _ = reader.read_u8(3); unique_id = None; set_id = None; }
        2 => { unique_id = None; set_id = None; }
        3 => { let _ = reader.read_u8(3); unique_id = None; set_id = None; }
        4 => { let _ = reader.read_u16(11); let _ = reader.read_u16(11); unique_id = None; set_id = None; }
        5 => { set_id = Some(reader.read_u16(12)); unique_id = None; }
        6 | 8 => {
            let _ = reader.read_u8(8); let _ = reader.read_u8(8);
            for _ in 0..6 { if reader.read_bit() == 1 { let _ = reader.read_u16(11); } }
            unique_id = None; set_id = None;
        }
        7 => { unique_id = Some(reader.read_u16(12)); set_id = None; }
        _ => {
            let _ = reader.read_u8(8); let _ = reader.read_u8(8);
            for _ in 0..6 { if reader.read_bit() == 1 { let _ = reader.read_u16(11); } }
            unique_id = None; set_id = None;
        }
    }

    if given_runeword { let _ = reader.read_u16(12); let _ = reader.read_u8(4); }
    if personalized {
        let char_bits: u8 = if version > 97 { 8 } else { 7 };
        for _ in 0..16 { let c = reader.read_u8(char_bits); if c == 0 { break; } }
    }

    // D2R RealmData
    if reader.read_bit() == 1 { reader.skip_bits(128); }

    // defense
    if is_armor || is_shield { let _ = reader.read_u16(11); }

    // durability - use stat table bit widths
    let mut maxd: u8 = 0;
    let mut curd: u8 = 0;
    if is_armor || is_shield {
        let md = reader.read_u16(table.get(73).save_bits.max(1)) as u8;
        maxd = md;
        let cd = reader.read_u16(table.get(72).save_bits.max(1)) as u8;
        curd = cd;
    }
    if is_weapon {
        let md = reader.read_u16(table.get(73).save_bits.max(1)) as u8;
        maxd = md;
        if md > 0 { curd = reader.read_u16(table.get(72).save_bits.max(1)) as u8; }
        else { reader.skip_bits(1); }
    }
    if version == 105 { reader.skip_bits(1); }
    if socketed || force_sockets { let _ = reader.read_u8(4); }
    let is_set = quality_bits == 5;
    let plist_flag: u8 = if is_set { reader.read_u8(5) } else { 0 };

    Ok((quality_bits, level, is_set, plist_flag, unique_id, set_id, maxd, curd))
}

pub(super) fn skip_non_simple_item_body_inner(
    reader: &mut BitReader,
    item_type: &str,
    _identified: bool,
    socketed: bool,
    personalized: bool,
    given_runeword: bool,
    _is_ear: bool,
    version: u8,
    force_sockets: bool,
    table: &StatTable,
) -> Result<u8, String> {
    let (quality_bits, _level, is_set, plist_flag, _unique_id, _set_id, _maxd, _curd) =
        read_pre_magic_fields(reader, item_type, socketed, personalized, given_runeword, version, force_sockets, table)?;

    // ★ 只有 Superior(3)和 Magic(4+)及以上品质才有 stat 列表。
    //   Normal(2)及以下的非简单物品不读 stat（无此数据，读则消耗过量位流）。
    if quality_bits >= 3 {
        read_magic_properties(reader, table)?;
    }

    if is_set {
        let mut plist = plist_flag;
        for _ in 0..5 {
            if (plist & 1) == 1 { read_magic_properties_limited(reader, 25)?; }
            plist >>= 1;
        }
    }
    if given_runeword {
        read_magic_properties_limited(reader, 25)?;
    }

    reader.skip_bits(8);
    Ok(quality_bits)
}
/// Each property: 9-bit stat ID → param (sP bits, if > 0) → value (sB bits, subtract sA).
/// Terminated by 0x1FF stat ID.
///
/// **★ D2SLib Items.cs logic ★**:  When the stat ID is unknown to our MAGICAL_PROPS
/// table (e.g. game v2.6+ official extensions, D2RMM mod-added stats), we MUST NOT
/// blindly skip a default number of bits — we scan forward for the 0x1FF terminator
/// at the next stat-boundary. This matches D2SLib's `ReadStatList` fallback.
fn read_magic_properties(reader: &mut BitReader, table: &StatTable) -> Result<(), String> {
    let mut iter = 0usize;
    loop {
        if reader.offset() >= reader.len_bits() {
            return Ok(());
        }

        let stat_start = reader.offset();
        let stat_id = reader.read_u16(9);
        if stat_id == 0x1FF {
            return Ok(());
        }

        iter += 1;
        if iter > 200 {
            return Ok(());
        }

        let prop = table.get(stat_id);
        let sb = prop.save_bits;
        let sp = prop.save_param_bits;
        let _sa = prop.save_add;
        let np = prop.num_sub_props.max(1) as usize;

        if sb == 0 && sp == 0 {
            if stat_id as usize >= table.len() {
                // Unknown stat beyond table — scan forward for 0x1FF
                reader.seek(stat_start);
                if scan_forward_for_terminator(reader) {
                    return Ok(());
                }
                return Ok(());
            }
            // Character-only stat — skip 1 bit
            reader.read_bit();
            continue;
        }

        for _ in 0..np {
            if sp > 0 {
                reader.skip_bits(sp as usize);
            }
            let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
            if vb <= 15 {
                reader.read_u16(vb);
            } else {
                reader.read_u32(vb);
            }
        }
    }
}

/// Scan forward from the current `reader.offset()` for the 0x1FF terminator (9 bits),
/// 1-bit step at a time, in a bounded budget (256 bits = 32 bytes).
/// On success: positions the reader immediately AFTER the consumed 0x1FF and returns true.
/// On failure: leaves the reader position untouched and returns false.
///
/// This mirrors D2SLib Items.cs `ReadStatList`: when a stat ID can't be
/// decoded, it advances the bit cursor toward the next potential terminator
/// rather than guessing bit width.
fn scan_forward_for_terminator(reader: &mut BitReader) -> bool {
    let start = reader.offset();
    let len_bits = reader.len_bits();
    let budget = 256usize;  // 32 bytes
    let mut probe = start;
    while probe + 9 <= len_bits && probe < start + budget {
        reader.seek(probe);
        if reader.read_u16(9) == 0x1FF {
            return true;  // reader is now positioned right after the 0x1FF 9-bit field
        }
        probe += 1;  // 1-bit step — same as D2SLib's safety search
    }
    // Could not find terminator within budget; restore original offset
    reader.seek(start);
    false
}

/// Same as read_magic_properties but with a max iteration limit for bonus blocks
/// (set bonuses, runeword bonuses) where the property list is short and 0x1FF should
/// appear quickly.
///
/// ★ Reuses `scan_forward_for_terminator` for unknown stats — same D2SLib fallback
/// logic as the main list reader.
fn read_magic_properties_limited(reader: &mut BitReader, max_iters: usize) -> Result<(), String> {
    for _ in 0..max_iters {
        if reader.offset() >= reader.len_bits() {
            return Ok(());
        }

        let stat_start = reader.offset();
        let stat_id = reader.read_u16(9);
        if stat_id == 0x1FF {
            return Ok(());
        }
        let sid = stat_id as usize;

        if sid >= MAGICAL_PROPS.len() {
            // ★ D2SLib Items.cs fallback ★
            reader.seek(stat_start);
            if scan_forward_for_terminator(reader) {
                return Ok(());
            }
            return Ok(());
        }

        let prop = &MAGICAL_PROPS[sid];
        let sb = prop.save_bits;
        let sp = prop.save_param_bits;
        let sa = prop.save_add;
        let np = prop.num_sub_props.max(1) as usize;

        if sb == 0 && sp == 0 {
            reader.read_bit();
            continue;
        }

        for _ in 0..np {
            if sp > 0 { reader.skip_bits(sp as usize); }
            let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
            if vb <= 15 {
                let raw = reader.read_u16(vb);
                if sa != 0 { let _adjusted = (raw as i32).wrapping_sub(sa); }
            } else {
                let raw = reader.read_u32(vb);
                if sa != 0 { let _adjusted = (raw as i32).wrapping_sub(sa); }
            }
        }
    }
    // Limit reached without finding 0x1FF — scan forward as best effort
    if !scan_forward_for_terminator(reader) {
        // Last resort: try to bail safely
    }
    Ok(())
}

// ── Phase 12: stat-parsing variants ─────────────────────────────

/// Read magic properties and return parsed `ItemStat` values.
/// Uses `StatTable` for bit-width lookups (instead of hardcoded MAGICAL_PROPS).
///
/// **★ keep in sync with `read_magic_properties`** — the skip path must
/// consume exactly the same bits so the parse path can advance identically.
pub fn parse_magic_properties_to_stats(
    reader: &mut BitReader,
    table: &StatTable,
) -> Result<Vec<ItemStat>, String> {
    let mut stats = Vec::new();
    let mut iter = 0usize;
    loop {
        if reader.offset() >= reader.len_bits() {
            return Ok(stats);
        }
        let _stat_start = reader.offset();
        let stat_id = reader.read_u16(9);
        if stat_id == 0x1FF {
            return Ok(stats);
        }

        iter += 1;
        let stat_start = reader.offset();
        if iter > 200 {
            return Ok(stats);
        }

        // Unknown stat (beyond table) — D2SLib Items.cs fallback:
        // roll back and scan for terminator instead of guessing bit width
        if stat_id as usize >= table.len() {
            reader.seek(stat_start);
            if scan_forward_for_terminator(reader) {
                return Ok(stats);
            }
            return Ok(stats);
        }

        let prop = table.get(stat_id);
        let sb = prop.save_bits;
        let sp = prop.save_param_bits;
        let sa = prop.save_add;
        let np = prop.num_sub_props.max(1) as usize;

        if sb == 0 && sp == 0 {
            reader.read_bit();
            continue;
        }

        // 主 stat (offset=0)
        let param_0 = if sp > 0 { reader.read_u32(sp) } else { 0 };
        let actual_bits_0 = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
        let raw_value_0 = reader.read_u32(actual_bits_0) as i64;
        let value_0 = raw_value_0 - sa as i64;
        stats.push(ItemStat { id: stat_id, param: param_0, value: value_0, ..Default::default() });

        // sub-props (offset 1..np): 每个 sub-prop 用自己的 stat_id 查位宽
        // e.g. lightmindam(50) + lightmaxdam(51): np=2, sub_id=51 uses 10 bits
        for offset in 1..np {
            let sub_id = stat_id.saturating_add(offset as u16);
            let sub_prop = table.get(sub_id);
            let ssb = sub_prop.save_bits;
            let ssp = sub_prop.save_param_bits;
            let ssa = sub_prop.save_add;
            let sub_param = if ssp > 0 { reader.read_u32(ssp) } else { 0 };
            let sub_bits = if ssb > 0 { ssb } else if ssp > 0 { 8 } else { 1 };
            let sub_raw = reader.read_u32(sub_bits) as i64;
            let sub_val = sub_raw - ssa as i64;
            stats.push(ItemStat { id: sub_id, param: sub_param, value: sub_val, ..Default::default() });
        }
    }
}

/// Same as `parse_magic_properties_to_stats` but with max iteration limit
/// for bonus blocks (set bonuses, runeword bonuses).
///
/// **★ keep in sync with `read_magic_properties_limited`** — the skip path
/// must consume exactly the same bits.
pub fn parse_magic_properties_limited_to_stats(
    reader: &mut BitReader,
    table: &StatTable,
    max_iters: usize,
) -> Result<Vec<ItemStat>, String> {
    let mut stats = Vec::new();
    for _ in 0..max_iters {
        if reader.offset() >= reader.len_bits() {
            return Ok(stats);
        }
        let stat_start = reader.offset();
        let stat_id = reader.read_u16(9);
        if stat_id == 0x1FF {
            return Ok(stats);
        }

        // Unknown stat (beyond table) — D2SLib Items.cs fallback
        if stat_id as usize >= table.len() {
            reader.seek(stat_start);
            if scan_forward_for_terminator(reader) {
                return Ok(stats);
            }
            return Ok(stats);
        }

        let prop = table.get(stat_id);
        let sb = prop.save_bits;
        let sp = prop.save_param_bits;
        let sa = prop.save_add;
        let np = prop.num_sub_props.max(1) as usize;

        if sb == 0 && sp == 0 {
            reader.read_bit();
            continue;
        }

        // 主 stat
        let param_0 = if sp > 0 { reader.read_u32(sp) } else { 0 };
        let actual_bits_0 = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
        let raw_0 = reader.read_u32(actual_bits_0) as i64;
        stats.push(ItemStat { id: stat_id, param: param_0, value: raw_0 - sa as i64, ..Default::default() });

        // sub-props: 每个用 sub_id 查各自位宽
        for offset in 1..np {
            let sub_id = stat_id.saturating_add(offset as u16);
            let sprop = table.get(sub_id);
            let ssp = sprop.save_param_bits;
            let ssb = sprop.save_bits;
            let ssa = sprop.save_add;
            let p = if ssp > 0 { reader.read_u32(ssp) } else { 0 };
            let bits = if ssb > 0 { ssb } else if ssp > 0 { 8 } else { 1 };
            let rv = reader.read_u32(bits) as i64;
            stats.push(ItemStat { id: sub_id, param: p, value: rv - ssa as i64, ..Default::default() });
        }
    }
    // limit reached — best-effort terminator scan
    let _ = scan_forward_for_terminator(reader);
    Ok(stats)
}

/// Parse a non-simple item's complete header and return all stat values.
///
/// Unlike `skip_non_simple_complete_header`, this actually reads and returns
/// the magic properties as `Vec<StatList>` (main + set bonuses + runeword bonuses).
///
/// **★ keep in sync with `skip_non_simple_item_body_inner` ★** — the mod-metadata
/// 8-bit skip must happen on both paths so reader positions match.
///
/// Returns (quality_raw_value, item_level, max_durability, current_durability, stat_lists).
pub fn parse_non_simple_complete_header(
    reader: &mut BitReader,
    item_type: &str,
    _identified: bool,
    socketed: bool,
    personalized: bool,
    given_runeword: bool,
    _is_ear: bool,
    version: u8,
    table: &StatTable,
) -> Result<(u8, u8, u8, u8, Vec<StatList>, Option<u16>, Option<u16>), String> {
    parse_non_simple_complete_header_with_options(
        reader,
        item_type,
        _identified,
        socketed,
        personalized,
        given_runeword,
        _is_ear,
        version,
        table,
        true,
    )
}


pub fn parse_non_simple_complete_header_with_options(
    reader: &mut BitReader,
    item_type: &str,
    _identified: bool,
    socketed: bool,
    personalized: bool,
    given_runeword: bool,
    _is_ear: bool,
    version: u8,
    table: &StatTable,
    skip_mod_metadata: bool,
) -> Result<(u8, u8, u8, u8, Vec<StatList>, Option<u16>, Option<u16>), String> {
    let (quality_bits, level, is_set, plist_flag, _unique_id, _set_id, maxd, curd) =
        read_pre_magic_fields(reader, item_type, socketed, personalized, given_runeword, version, false, table)?;
    let mut stat_lists: Vec<StatList> = Vec::new();

    // ── 按品质读取属性表 ──
    //
    // D2R 格式：只有 Q≥3（Superior/Magic/Set/Rare/Unique/Crafted）才有词缀属性表。
    // Normal(2) 物品无 stat 列表，读取会导致位流错位。
    if quality_bits >= 3 {
        let main_stats = parse_magic_properties_to_stats(reader, table)?;
        stat_lists.push(StatList { stats: main_stats });

        if is_set {
            let mut plist = plist_flag;
            for _ in 0..5 {
                if (plist & 1) == 1 {
                    let bonus = parse_magic_properties_limited_to_stats(reader, table, 25)?;
                    stat_lists.push(StatList { stats: bonus });
                }
                plist >>= 1;
            }
        }

        if given_runeword {
            let bonus = parse_magic_properties_limited_to_stats(reader, table, 25)?;
            stat_lists.push(StatList { stats: bonus });
        }
    }

    if skip_mod_metadata {
        reader.skip_bits(8);
    }

    Ok((quality_bits, level, maxd, curd, stat_lists, _unique_id, _set_id))
}

#[cfg(test)]
mod tests {
    use super::lookup_item_category;

    #[test]
    fn test_lookup_item_category_marks_grimoire_family_as_shield() {
        assert_eq!(lookup_item_category("wa1"), (false, false, true));
        assert_eq!(lookup_item_category("wae"), (false, false, true));
        assert_eq!(lookup_item_category("waf"), (false, false, true));
    }
}
