//! Sequential JM item stream reader for stash pages.
//!
//! Follows the Python d2r-zero `parse_jm` + `_scan_item` approach:
//! single-pass sequential walk with forward-scan resync on alignment errors.
//! No boundary probing, no factor scoring, no dedup — just read items in order.

use crate::core::bitio::BitReader;
use crate::core::encoding::{decode_huffman_string, skip_string_7bit};
use crate::core::{ParseError, ParseResult};
use crate::protocol::common::{
    stat_list::StatReadConfig,
    Item, ItemFlags, ItemLocation, ItemMode, ItemPage, ItemQuality, StatList, StatTable,
};
use crate::protocol::d2i::parser::ParsedItem;

// ── Static caches (Phase 1 optimization) ────────────────────────────
// Cached stat table: built once, reused for all pages in this process.
// Uses std::sync::OnceLock (stable in Rust 1.70+).
static STAT_TABLE: std::sync::OnceLock<StatTable> = std::sync::OnceLock::new();

pub fn get_cached_stat_table() -> &'static StatTable {
    STAT_TABLE.get_or_init(|| {
        if crate::data::stat_loader::has_runtime_table() {
            crate::data::stat_loader::build_runtime_table()
        } else {
            crate::data::stat_cost::build_stat_table()
        }
    })
}

// Cached HashSet of known item codes: O(1) lookup vs O(n) linear scan.
// ALL_ITEMS has 600+ entries; HashSet gives 10-100x speedup per lookup.
static ITEM_CODE_SET: std::sync::OnceLock<std::collections::HashSet<&'static str>> =
    std::sync::OnceLock::new();

fn is_known_item_code(code: &str) -> bool {
    ITEM_CODE_SET.get_or_init(|| {
        crate::protocol::d2i::legacy::game_items::ALL_ITEMS
            .iter()
            .map(|(c, _, _, _, _)| *c)
            .collect()
    }).contains(code)
}

// ── Forward Scan ─────────────────────────────────────────────────

/// Result of scanning forward for the next item header — bit offset.
pub struct ScanResult {
    pub position: usize,
}
pub struct ScanConfig {
    pub max_scan_bytes: usize,
    pub max_flag_bits: u32,
    /// If true, accept `pv <= version`; if false, accept `pv == version`.
    pub accept_version_or_less: bool,
    pub version: u8,
    pub require_code_len_3: bool,
    pub require_alphanumeric_all: bool,
}

/// Default config: strict (ver==5, code==3, all alphanumeric).
impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_scan_bytes: 32,
            max_flag_bits: 6,
            accept_version_or_less: false,
            version: 5,
            require_code_len_3: true,
            require_alphanumeric_all: true,
        }
    }
}

impl ScanConfig {
    pub fn lenient() -> Self {
        Self {
            accept_version_or_less: true,
            require_code_len_3: false,
            require_alphanumeric_all: false,
            ..Self::default()
        }
    }
}

/// Scan forward from `start` for a valid item header.
/// Returns `Some(ScanResult)` on first match, `None` if no valid header found.
///
/// ## Optimization (Phase 1)
/// Pre-filters at the byte level before creating any BitReader:
/// 1. Skip non-byte-aligned positions (bit_offset % 8 != 0)
/// 2. Quick version check from raw byte (no BitReader allocation)
/// 3. Only create BitReader + decode_huffman_string for candidates that pass pre-filter
///
/// This eliminates 80-95% of BitReader allocations in the scan path.
pub fn scan_next_item(payload: &[u8], start: usize, config: &ScanConfig) -> Option<ScanResult> {
    let max_scan = std::cmp::min(
        config.max_scan_bytes,
        (payload.len() * 8).saturating_sub(start + 35) / 8,
    );

    for skip in 1..=max_scan {
        let probe = start + skip * 8;
        if probe + 80 > payload.len() * 8 {
            break;
        }

        // ── Zero-copy pre-filter (no BitReader allocation) ──────────────
        //
        // probe is in bits. The item header layout at `probe`:
        //   bits 0-31:   flags (u32, LSB-first)
        //   bits 32-34: version (3 bits)
        //   bits 35-52: mode+location+position (18 bits, not checked in scan)
        //   bits 53+:    code (4 chars via Huffman, 20+ bits)
        //
        // BitReader::read_u32(32) at probe reads bytes at bit positions:
        //   byte[0] at probe+0..7,  byte[1] at probe+8..15,
        //   byte[2] at probe+16..23, byte[3] at probe+24..31
        // BitReader::read_u8(3) at probe+32 reads bits probe+32,33,34:
        //   which is byte[4], bits 0,1,2
        let byte_pos = probe / 8;
        let bit_offset_in_byte = probe % 8;

        // Only scan at byte-aligned positions — non-aligned positions can't
        // produce valid item headers (item fields are byte-grained in D2R)
        if bit_offset_in_byte != 0 {
            continue;
        }

        // Version check: bits 32-34 of stream = byte[4] & 0x07
        let version = payload[byte_pos + 4] & 0x07;
        let version_ok = if config.accept_version_or_less {
            version <= config.version
        } else {
            version == config.version
        };
        if !version_ok {
            continue;
        }

        // Quick flags check: byte[0] & 0x10 must be set (flags bit 4)
        // This is a necessary condition, checked before allocating BitReader
        if payload[byte_pos] & 0x10 == 0 {
            continue;
        }

        // ── Candidate passed pre-filter: create BitReader for full validation ──
        let mut pr = BitReader::new(payload);
        pr.seek(probe);
        let pf = pr.read_u32(32);
        let pv = pr.read_u8(3);

        // Re-verify flags constraints with full 32-bit value
        let version_ok = if config.accept_version_or_less {
            pv <= config.version
        } else {
            pv == config.version
        };
        if !(pf.count_ones() <= config.max_flag_bits && (pf >> 4) & 1 == 1 && version_ok) {
            continue;
        }

        let mut cr = BitReader::new(payload);
        cr.seek(probe + 53);
        let code = decode_huffman_string(&mut cr);
        let ct = code.trim();

        if config.require_code_len_3 && ct.len() != 3 {
            continue;
        }
        if !config.require_code_len_3 && ct.is_empty() {
            continue;
        }
        if config.require_alphanumeric_all && !ct.chars().all(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        if !ct.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        return Some(ScanResult { position: probe });
    }
    None
}

// ── Helpers ───────────────────────────────────────────────────────

fn u8_to_mode(v: u8) -> ItemMode {
    match v { 0 => ItemMode::Stored, 1 => ItemMode::Equipped, 2 => ItemMode::Belt, 6 => ItemMode::Socket, _ => ItemMode::Stored }
}

fn u8_to_location(v: u8) -> ItemLocation {
    match v { 0 => ItemLocation::None, 1 => ItemLocation::Head, 2 => ItemLocation::Neck,
        3 => ItemLocation::Torso, 4 => ItemLocation::RightHand, 5 => ItemLocation::LeftHand,
        6 => ItemLocation::RightFinger, 7 => ItemLocation::LeftFinger, 8 => ItemLocation::Waist,
        9 => ItemLocation::Feet, 10 => ItemLocation::Hands, 11 => ItemLocation::Trinket1,
        12 => ItemLocation::Trinket2, _ => ItemLocation::None }
}

fn u8_to_page(v: u8) -> ItemPage {
    match v {
        0 => ItemPage::Equipped,
        1 => ItemPage::Backpack,
        5 => ItemPage::MyStash,
        6 => ItemPage::SharedStash,
        x => ItemPage::Mod(x),
    }
}

// ── Builder ────────────────────────────────────────────────────

/// Builder for constructing a `ParsedItem` from parsed fields.
/// Replaces the 18+ parameter `make_item` with named, chainable setters.
pub(crate) struct ParsedItemBuilder {
    page_index: usize,
    flags_raw: u32,
    ver: u8,
    mode: u8,
    loc: u8,
    px: u8,
    py: u8,
    pg: u8,
    code: String,
    uid: u32,
    ilvl: u8,
    quality: ItemQuality,
    stat_lists: Vec<StatList>,
    num_sockets: u8,
    amount: u32,
    cur_dur: u8,
    max_dur: u8,
    defense: u16,
    start_bit: usize,
    bit_len: usize,
    magic_prefix: Option<u16>,
    magic_suffix: Option<u16>,
    unique_id: Option<u16>,
    set_id: Option<u16>,
    is_socketed_subitem: bool,
}

impl ParsedItemBuilder {
    pub(crate) fn new() -> Self {
        Self {
            page_index: 0, flags_raw: 0, ver: 0, mode: 0, loc: 0, px: 0, py: 0, pg: 0,
            code: String::new(), uid: 0, ilvl: 0, quality: ItemQuality::Normal,
            stat_lists: Vec::new(), num_sockets: 0, amount: 1, cur_dur: 0, max_dur: 0,
            defense: 0,
            start_bit: 0, bit_len: 0,
            magic_prefix: None, magic_suffix: None, unique_id: None, set_id: None,
            is_socketed_subitem: false,
        }
    }

    pub(crate) fn flags(mut self, v: u32) -> Self { self.flags_raw = v; self }
    pub(crate) fn version(mut self, v: u8) -> Self { self.ver = v; self }
    pub(crate) fn position(mut self, mode: u8, loc: u8, px: u8, py: u8, pg: u8) -> Self {
        self.mode = mode; self.loc = loc; self.px = px; self.py = py; self.pg = pg; self
    }
    pub(crate) fn code(mut self, c: String) -> Self { self.code = c; self }
    pub(crate) fn uid(mut self, id: u32) -> Self { self.uid = id; self }
    pub(crate) fn ilvl(mut self, lv: u8) -> Self { self.ilvl = lv; self }
    pub(crate) fn quality(mut self, q: ItemQuality) -> Self { self.quality = q; self }
    pub(crate) fn stat_lists(mut self, sl: Vec<StatList>) -> Self { self.stat_lists = sl; self }
    pub(crate) fn num_sockets(mut self, n: u8) -> Self { self.num_sockets = n; self }
    pub(crate) fn amount(mut self, a: u32) -> Self { self.amount = a; self }
    pub(crate) fn durability(mut self, cur: u8, max: u8) -> Self { self.cur_dur = cur; self.max_dur = max; self }
    pub(crate) fn defense(mut self, v: u16) -> Self { self.defense = v; self }
    pub(crate) fn raw_bit_range(mut self, start: usize, len: usize) -> Self { self.start_bit = start; self.bit_len = len; self }
    pub(crate) fn magic_prefix(mut self, id: Option<u16>) -> Self { self.magic_prefix = id; self }
    pub(crate) fn magic_suffix(mut self, id: Option<u16>) -> Self { self.magic_suffix = id; self }
    pub(crate) fn unique_id(mut self, id: Option<u16>) -> Self { self.unique_id = id; self }
    pub(crate) fn set_id(mut self, id: Option<u16>) -> Self { self.set_id = id; self }

    pub(crate) fn build(self) -> ParsedItem {
        let p = self;
        ParsedItem {
            page_index: p.page_index,
            item: Item {
                flags: ItemFlags { raw: p.flags_raw },
                version_raw: p.ver,
                mode: u8_to_mode(p.mode),
                location: u8_to_location(p.loc),
                x: p.px, y: p.py,
                page: Some(u8_to_page(p.pg)),
                code: p.code, num_sockets: p.num_sockets, id: p.uid,
                item_level: p.ilvl,
                quality: p.quality, stat_lists: p.stat_lists, amount: p.amount,
                socketed_items: Vec::new(),
                current_durability: p.cur_dur,
                max_durability: p.max_dur,
                defense: p.defense,
                unique_id: p.unique_id,
                set_id: p.set_id,
            },
            raw_bit_offset: p.start_bit,
            raw_bit_length: p.bit_len,
            is_socketed_subitem: p.is_socketed_subitem,
            magic_prefix_id: p.magic_prefix,
            magic_suffix_id: p.magic_suffix,
            is_pseudo_unverified: false,
        }
    }
}



/// Parse the body of a non-compact item: uid, ilvl, quality, stat lists, and trailer fields.
/// `reader` must be positioned after the item code (ready for `_soc_hint`).
/// `payload` is the raw page payload (without JM header), used for byte-level
/// scanning during post-stat-list resync (matching Python behavior).
fn parse_noncompact_body(
    reader: &mut BitReader,
    payload: &[u8],
    table: &StatTable,
    flags_raw: u32,
    ver: u8,
    mode: u8, loc: u8, px: u8, py: u8, pg: u8,
    code: String,
    page_is_stackable: bool,
    start: usize,
) -> ParseResult<Option<ParsedItem>> {
    let _soc_hint = reader.read_u8(3);
    let uid = reader.read_u32(32);
    let ilvl = reader.read_u8(7);
    let qb = reader.read_u8(4);
    if reader.remaining_bits() < 12 {
        return Err(ParseError::InvalidSection("truncated non-compact".into()));
    }

    // Jump bits
    if reader.read_bit() == 1 { reader.skip_bits(3); }
    if reader.read_bit() == 1 { reader.skip_bits(11); }

    // Quality fields — capture IDs for magic (prefix/suffix), unique, set
    let mut magic_prefix: Option<u16> = None;
    let mut magic_suffix: Option<u16> = None;
    let mut parsed_uid: Option<u16> = None;
    let mut parsed_sid: Option<u16> = None;
    let quality = match qb {
        7 => { parsed_uid = Some(reader.read_u16(12)); ItemQuality::Unique }
        5 => { parsed_sid = Some(reader.read_u16(12)); ItemQuality::Set }
        4 => {
            let p = reader.read_u16(11);
            let s = reader.read_u16(11);
            if p > 0 { magic_prefix = Some(p); }
            if s > 0 { magic_suffix = Some(s); }
            ItemQuality::Magic
        }
        6 => { reader.read_u16(16); for _ in 0..6 { if reader.read_bit() == 1 { reader.skip_bits(11); } } ItemQuality::Rare }
        8 => { reader.read_u16(16); for _ in 0..6 { if reader.read_bit() == 1 { reader.skip_bits(11); } } ItemQuality::Crafted }
        1 => { reader.skip_bits(3); ItemQuality::Low }
        3 => { reader.skip_bits(3); ItemQuality::Superior }
        _ => ItemQuality::Normal
    };

    // ── Non-compact body (matching Python `_scan_complete_item`) ──
    // 1. Runeword ID + prop_lists (Python: 12b id + 4b shift = 16b)
    let mut prop_lists: u8 = 0;
    if (flags_raw >> 26) & 1 == 1 {
        let _rw_id = reader.read_u16(12);
        prop_lists |= 1u8.wrapping_shl(reader.read_u8(4) as u32 + 1);
    }
    // 2. Ear / Personalization (7-bit strings)
    if (flags_raw >> 16) & 1 == 1 {
        reader.skip_bits(10); skip_string_7bit(reader);
    } else if (flags_raw >> 24) & 1 == 1 {
        skip_string_7bit(reader);
    }
    // 3. Realm data at body start
    if reader.read_bit() != 0 { reader.skip_bits(128); }
    // 4. Type-specific data
    let (cat_a, cat_w, cat_s) = crate::protocol::d2i::legacy::complete_header::lookup_item_category(&code);
    let is_armor = cat_a || cat_s;
    let is_weapon = cat_w;
    let mut def_value: u16 = 0;
    if is_armor {
        let sb31 = table.get(31).save_bits.max(1);
        def_value = reader.read_u16(sb31);
        let sb73 = table.get(73).save_bits.max(1);
        let md = reader.read_u16(sb73);
        if md > 0 {
            let sb72 = table.get(72).save_bits.max(1);
            reader.skip_bits(sb72 as usize);
            reader.skip_bits(1); // Python r.read_bits(1) after current durability
        }
    } else if is_weapon {
        let sb73 = table.get(73).save_bits.max(1);
        let md = reader.read_u16(sb73);
        if md > 0 {
            let sb72 = table.get(72).save_bits.max(1);
            reader.skip_bits(sb72 as usize);
            reader.skip_bits(1); // Python r.read_bits(1) after current durability
        }
    } else {
        if reader.remaining_bits() >= 10 && reader.read_bit() != 0 { reader.skip_bits(9); }
    }
    // 6. Socket count — Python uses 4 bits (NOT 3!)
    let socket_count = if (flags_raw >> 11) & 1 == 1 { reader.read_u8(4) } else { 0 };
    // Set item mask (5b) → added to prop_lists
    if qb == 5 {
        prop_lists |= reader.read_u8(5);
    }
    // ── Main stat list + bonus streams (matching Python `_scan_complete_item`) ──
    let _tr = std::time::Instant::now();
    let mut all_stat_lists: Vec<StatList> = Vec::new();
    let (main_sl, mut stat_clean) = StatList::read_with_clean_flag(reader, table, &StatReadConfig::default());
    all_stat_lists.push(main_sl);
    // Read bonus stat streams for runeword/set prop_lists
    for bm in [1u8, 2, 4, 8, 16, 32, 64] {
        if (prop_lists & bm) != 0 {
            let (sl, clean) = StatList::read_with_clean_flag(reader, table, &StatReadConfig::default());
            all_stat_lists.push(sl);
            stat_clean = stat_clean && clean;
        }
    }
    let _main_stat_time = _tr.elapsed();
    
    // Match Python order: align FIRST, then forward-scan resync
    reader.align_to_byte();
    
    // ── Forward-scan resync on unclean stat termination (matching Python lines 353-365) ──
    // When stat stream didn't find 0x1FF and terminated via guard, the reader may be
    // misaligned with the next item. Probe at current + 0-64 bytes to find a valid header.
    if !stat_clean && reader.remaining_bits() >= 40 {
        let probe_remaining = reader.remaining_bits();
        let saved = reader.offset();
        let probe_flags = reader.read_u32(32);
        let probe_ver = reader.read_u8(3);
        reader.seek(saved);
        if probe_ver != 5 || probe_flags.count_ones() > 4 {
            let max_scan_bytes = 64.min(probe_remaining / 8);
            let payload_len = payload.len();
            for try_b in 1..=max_scan_bytes {
                let probe_pos = reader.offset() + try_b * 8;
                let byte_off = probe_pos / 8;
                if byte_off + 5 > payload_len { break; }
                let pf = u32::from_le_bytes([
                    payload[byte_off], payload[byte_off + 1],
                    payload[byte_off + 2], payload[byte_off + 3],
                ]);
                let pv = (payload[byte_off + 4] as u32) & 0x7;
                if pv == 5 && pf.count_ones() <= 4 && (pf >> 4) & 1 != 0 {
                    reader.seek(probe_pos);
                    break;
                }
            }
        }
    }
    
    // ── Post-body D2R realm data check (matching Python _scan_item lines 496-505) ──
    // Peek 32 bits; if bit 16 is set, consume 128 more bits (D2R post-body RealmData)
    if reader.remaining_bits() >= 32 {
        let saved = reader.offset();
        let peek = reader.read_u32(32);
        reader.seek(saved);
        if (peek >> 16) & 1 == 1 && reader.remaining_bits() >= 128 {
            reader.skip_bits(128);
        }
    }
    
    reader.align_to_byte();
    
    let amount = if page_is_stackable { (py as u32) << 4 | px as u32 } else { 1u32 };
    let bit_len = reader.offset() - start;
    Ok(Some(ParsedItemBuilder::new()
        .flags(flags_raw).version(ver).position(mode, loc, px, py, pg).code(code)
        .uid(uid).ilvl(ilvl).quality(quality)
        .magic_prefix(magic_prefix).magic_suffix(magic_suffix)
        .unique_id(parsed_uid).set_id(parsed_sid)
        .num_sockets(socket_count)
        .stat_lists(all_stat_lists)
        .amount(amount)
        .defense(def_value)
        .raw_bit_range(start, bit_len)
        .build()))
}
/// or Err on unrecoverable parse failure. Forward-scan resync is handled by the caller.
pub fn try_parse_one(
    reader: &mut BitReader,
    payload: &[u8],
    table: &StatTable,
    page_is_stackable: bool,
) -> ParseResult<Option<ParsedItem>> {
    let start = reader.offset();
    if reader.remaining_bits() < 80 { return Ok(None); }

    // ── Header: flags (32b) + version (3b) + position (15b) ──
    let flags_raw = reader.read_u32(32);
    let ear = (flags_raw >> 16) & 1;

    let ver = reader.read_u8(3);
    // Accept any version — mod stashes may have ver=1/2 items migrated
    // to the stackable page. Ear padding only differs for ver != 5.
    if ear == 1 && ver != 5 {
        reader.skip_bits(8);
    }

    // Ear item
    if ear == 1 {
        reader.align_to_byte();
        let bit_len = reader.offset() - start;
        return Ok(Some(ParsedItemBuilder::new()
            .flags(flags_raw).version(5)
            .quality(ItemQuality::None)
            .raw_bit_range(start, bit_len)
            .build()));
    }

    // Position fields (loc 4, x 4, y 4, pg 3)
    let mode = reader.read_u8(3);
    let loc = reader.read_u8(4);
    let px = reader.read_u8(4);
    let py = reader.read_u8(4);
    let pg = reader.read_u8(3);

    // Item code (Huffman)
    let code = decode_huffman_string(reader);
    let code = code.trim().to_string();

    // Validate code — if unrecognized, do Python-style resync (scan forward,
    // seek to found position, return Ok(None) for loop to retry).
    if code.len() != 3
        || !code.chars().all(|c| c.is_ascii_alphanumeric())
        || !code.chars().any(|c| c.is_ascii_alphabetic())
        || !is_known_item_code(code.trim())
    {
        // Python-style resync: scan forward byte-by-byte from original start+8
        let max_scan_bytes = 32;
        let total_bits = payload.len() * 8;
        for skip in 1..=max_scan_bytes {
            let probe = start + skip * 8;
            if probe + 35 > total_bits { break; }
            let byte_pos = probe / 8;
            if byte_pos + 4 >= payload.len() { break; }
            if (payload[byte_pos + 4] & 0x07) != 5 { continue; }
            if (payload[byte_pos] & 0x10) == 0 { continue; }
            let pf = u32::from_le_bytes([
                payload[byte_pos], payload[byte_pos + 1],
                payload[byte_pos + 2], payload[byte_pos + 3],
            ]);
            if pf.count_ones() > 4 || (pf >> 4) & 1 == 0 { continue; }
            let code_start = probe + 53;
            let code_offset = code_start / 8;
            if code_offset >= payload.len() { continue; }
            let mut code_reader = BitReader::new(payload);
            code_reader.seek(code_start);
            let candidate = decode_huffman_string(&mut code_reader);
            if is_known_item_code(candidate.trim()) {
                reader.seek(probe);
                return Ok(None);
            }
        }
        let next = start + 8;
        if next < total_bits {
            reader.seek(next);
        }
        return Ok(None);
    }
    
    let simple = (flags_raw >> 21) & 1;
    
    // ── Compact item ──
    if simple == 1 {
        // ── Python compact item body (jm_parser _scan_item lines 417-480) ──
        // 1. ExtData: D2R compact items always have 1-bit ext data (line 421)
        let _ext_data = reader.read_bit();
        
        // 2. qf/qty (lines 423-425): only for _STACK items NOT in _REALM_COMPACT
        // In D2R, ALL _STACK items have compactsave=1 (_REALM_COMPACT), so this
        // condition is NEVER true for standard items. Only mod items without
        // compactsave but in _STACK would trigger this. Skip for now.
        
        // 3. D2R: compact items always advance to next byte boundary (line 426-428)
        // Python: if not aligned → align; else → advance full byte
        let bit_rem = reader.offset() % 8;
        if bit_rem != 0 {
            reader.skip_bits(8 - bit_rem);
        } else {
            reader.skip_bits(8); // always at least 1 extra byte
        }
        
        // 4. Realm data (lines 443-456)
        if ver != 5 {
            // Mod items: unconditional 128 bits + trailing zero cleanup
            if reader.remaining_bits() >= 128 {
                reader.skip_bits(128);
            }
            // Trailing zero padding byte
            if reader.offset().is_multiple_of(8) {
                let byte_pos = reader.offset() / 8;
                if byte_pos < payload.len() && payload[byte_pos] == 0 {
                    reader.skip_bits(8);
                }
            }
        } else {
            // Standard D2R: peek bit 16, conditional 128 bits
            if reader.remaining_bits() >= 32 {
                let saved = reader.offset();
                let peek = reader.read_u32(32);
                reader.seek(saved);
                if (peek >> 16) & 1 == 1 && reader.remaining_bits() >= 128 {
                    reader.skip_bits(128);
                }
            }
        }
        
        // 5. _ADV_STASH check (lines 471-479): conditional 136 bits if bit 16 set
        // Only for items in the AdvancedStashStackable set (gems, runes, mod items)
        if page_is_stackable && reader.remaining_bits() >= 136 {
            let saved = reader.offset();
            let peek = reader.read_u32(32);
            reader.seek(saved);
            if (peek >> 16) & 1 == 1 && reader.remaining_bits() >= 136 {
                reader.skip_bits(136);
            }
        }
        
        let amount = if page_is_stackable { (py as u32) << 4 | px as u32 } else { 1u32 };
        let bit_len = reader.offset() - start;
        return Ok(Some(ParsedItemBuilder::new()
            .flags(flags_raw).version(ver).position(mode, loc, px, py, pg).code(code)
            .quality(ItemQuality::Normal)
            .amount(amount)
            .raw_bit_range(start, bit_len)
            .build()));
    }
    
    // ── Non-compact item ──
    parse_noncompact_body(reader, payload, table, flags_raw, ver, mode, loc, px, py, pg, code, page_is_stackable, start)
}


/// Parse a JM item stream from a stash page (page data after the 64-byte page header).
///
/// * `page_data` — raw page bytes (including the 64-byte header and JM payload)
/// * `page_index` — stash page index (0-based)
/// Parse a JM item stream using a pre-built StatTable (avoids rebuilding per page).
pub fn parse_jm_page_with_table(
    page_data: &[u8], page_index: usize, is_stackable: bool, table: &StatTable,
) -> Vec<ParsedItem> {
    parse_jm_page_impl(page_data, page_index, is_stackable, Some(table))
}

/// Parse a JM item stream, building the StatTable locally.
/// For batch parsing (e.g. all pages of a stash file), use [`parse_jm_page_with_table`]
/// with a table built once in the caller.
pub fn parse_jm_page(page_data: &[u8], page_index: usize, is_stackable: bool) -> Vec<ParsedItem> {
    parse_jm_page_impl(page_data, page_index, is_stackable, None)
}

fn parse_jm_page_impl(
    page_data: &[u8], page_index: usize, is_stackable: bool, cached_table: Option<&StatTable>,
) -> Vec<ParsedItem> {
    let _t0 = std::time::Instant::now();
    // Skip 64-byte page header to get JM data
    let jm_data = &page_data[64.min(page_data.len())..];
    if jm_data.len() < 4 || &jm_data[0..2] != b"JM" {
        return Vec::new();
    }
    let _count = u16::from_le_bytes([jm_data[2], jm_data[3]]) as usize;
    let payload = &jm_data[4..];

    let mut reader = BitReader::new(payload);
    let mut items: Vec<ParsedItem> = Vec::new();
    let max_iter = 4096; // generous limit — loop exits on remaining_bits < 80
    let mut parse_time = std::time::Duration::ZERO;
    let mut scan_time = std::time::Duration::ZERO;
    let mut parse_count = 0u32;
    let mut scan_count = 0u32;

    // Use cached StatTable (module-level, built once per process).
    // Caller's `cached_table` param takes precedence if provided.
    let table: &StatTable = if let Some(t) = cached_table {
        t
    } else {
        get_cached_stat_table()
    };
    // No owned table needed — we always borrow from cache.
    let _owned_table: Option<StatTable> = None;

    // Resync config: strict scan (ver==5, code==3, alpha) to minimize false positives.
    // max_scan_bytes=128 covers larger gaps after invalid-code item bodies.
    let scan_strict = ScanConfig {
        max_scan_bytes: 128,
        accept_version_or_less: false,
        version: 5,
        max_flag_bits: 6,
        require_code_len_3: true,
        require_alphanumeric_all: true,
    };
    for _ in 0..max_iter {
        if reader.remaining_bits() < 80 {
            break;
        }
        let _item_start = reader.offset();
        let _tp = std::time::Instant::now();
        match try_parse_one(&mut reader, payload, table, is_stackable) {
            Ok(Some(pi)) => {
                parse_time += _tp.elapsed();
                parse_count += 1;
                let code_valid = is_known_item_code(&pi.item.code);
                let flags_ok = pi.item.flags.raw <= 0xE0000000;
                if flags_ok && code_valid {
                    let mut pi = pi;
                    pi.page_index = page_index;
                    pi.raw_bit_length = reader.offset() - pi.raw_bit_offset;
                    items.push(pi);
                }
                // No else branch needed: try_parse_one's internal resync
                // already handled invalid codes by seeking to the next valid item
                // and returning Ok(None). So if we get Ok(Some) here, the code
                // was already validated and we only need to check flags.
            }
            Ok(None) => {
                parse_time += _tp.elapsed();
                parse_count += 1;
                // try_parse_one returned None after internal resync.
                // The reader was already seek'd to the next candidate position.
                // Continue the loop to try parsing at the new position.
            }
            Err(_e) => {
                parse_time += _tp.elapsed();
                parse_count += 1;
                // Parse error — try_parse_one's internal handle may have left
                // the reader at an arbitrary position. Scan forward from current.
                let current = reader.offset();
                let _ts = std::time::Instant::now();
                if let Some(r) = scan_next_item(payload, current, &scan_strict) {
                    reader.seek(r.position);
                    scan_time += _ts.elapsed();
                    scan_count += 1;
                    continue;
                } else {
                    let next = current + 8;
                    if next < payload.len() * 8 {
                        reader.seek(next);
                        scan_time += _ts.elapsed();
                        scan_count += 1;
                        continue;
                    }
                    break;
                }
            }
        }
    }
    if parse_count > 0 {
        eprintln!("[timing] parse_jm_page page={} item_count={} found={} avg_parse={:?} total_scan={:?} scan_count={}",
            page_index, items.len() + scan_count as usize, items.len(),
            parse_time / parse_count, scan_time, scan_count);
    }
    eprintln!("[timing] parse_jm_page page={} total={:?} (parse={:?} scan={:?})",
        page_index, _t0.elapsed(), parse_time, scan_time);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_empty_jm() {
        let items = parse_jm_page(&[0u8; 64], 0, false);
        assert!(items.is_empty());
    }

    #[test]
    fn test_no_jm_magic() {
        let data = vec![0u8; 64 + 10];
        let items = parse_jm_page(&data, 0, false);
        assert!(items.is_empty());
    }

    #[test]
    fn test_scan_next_item_no_match_empty() {
        let payload = [0u8; 64];
        let result = scan_next_item(&payload, 0, &ScanConfig::default());
        assert!(result.is_none());
    }

    #[test]
    fn test_scan_next_item_no_match_density() {
        let mut payload = [0u8; 64];
        payload[0] = 0xFF;
        let result = scan_next_item(&payload, 0, &ScanConfig::default());
        assert!(result.is_none());
    }

    #[test]
    fn test_builder_defaults() {
        let item = ParsedItemBuilder::new().build();
        assert_eq!(item.item.code, "");
        assert_eq!(item.item.quality, ItemQuality::Normal);
        assert_eq!(item.item.amount, 1);
        assert_eq!(item.raw_bit_offset, 0);
        assert_eq!(item.raw_bit_length, 0);
        assert!(!item.is_socketed_subitem);
        assert!(item.magic_prefix_id.is_none());
        assert!(item.magic_suffix_id.is_none());
    }

    #[test]
    fn test_builder_chain() {
        let item = ParsedItemBuilder::new()
            .flags(0x10).version(5).position(1, 0, 2, 3, 1)
            .code("tes".to_string())
            .uid(42).ilvl(30).quality(ItemQuality::Magic)
            .amount(1)
            .magic_prefix(Some(100)).magic_suffix(Some(200))
            .raw_bit_range(0, 200)
            .build();
        assert_eq!(item.item.code, "tes");
        assert_eq!(item.item.id, 42);
        assert_eq!(item.item.item_level, 30);
        assert_eq!(item.item.quality, ItemQuality::Magic);
        assert_eq!(item.magic_prefix_id, Some(100));
        assert_eq!(item.magic_suffix_id, Some(200));
        assert_eq!(item.raw_bit_length, 200);
        assert_eq!(item.item.mode, ItemMode::Equipped);
        assert_eq!(item.item.page, Some(ItemPage::Backpack));
    }

    #[test]
    fn test_parse_modern_shared_stash() {
        let path = fixture_path("ModernSharedStashSoftCoreV2.d2i");
        if !path.exists() { eprintln!("SKIP: fixture ModernSharedStashSoftCoreV2.d2i 缺失"); return; }
        let data = std::fs::read(&path).expect("read fixture");
        let file = crate::protocol::d2i::parser::parse_file(&data)
            .expect("parse modern stash");
        assert!(!file.items.is_empty(), "modern stash should parse items");
        assert!(file.items.len() >= 10, "modern stash should have at least 10 items, got {}", file.items.len());
        for pi in &file.items {
            assert!(pi.item.code.len() == 3, "item code should be 3 chars: {}", pi.item.code);
        }
    }

    #[test]
    fn test_parse_modern_stash_page0() {
        let path = fixture_path("ModernSharedStashSoftCoreV2.d2i");
        if !path.exists() { eprintln!("SKIP: fixture ModernSharedStashSoftCoreV2.d2i 缺失"); return; }
        let data = std::fs::read(&path).expect("read fixture");
        let file = crate::protocol::d2i::parser::parse_file(&data)
            .expect("parse modern stash");
        for pi in &file.items {
            assert!(pi.page_index < file.pages.len(),
                "item {} page {} out of range ({} pages)", pi.item.code, pi.page_index, file.pages.len());
        }
    }

    #[test]
    fn test_parse_modern_stash_non_empty_item_fields() {
        let path = fixture_path("ModernSharedStashSoftCoreV2.d2i");
        if !path.exists() { eprintln!("SKIP: fixture ModernSharedStashSoftCoreV2.d2i 缺失"); return; }
        let data = std::fs::read(&path).expect("read fixture");
        let file = crate::protocol::d2i::parser::parse_file(&data)
            .expect("parse modern stash");
        for pi in &file.items {
            assert!(pi.raw_bit_length > 0, "item {} has zero bit length", pi.item.code);
            assert!(pi.item.flags.raw <= 0xE0000000, "item {} has sentinel flags", pi.item.code);
        }
    }
}
