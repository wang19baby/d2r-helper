//! D2I parser — simplified sequential JM reader.
//!
//! Delegates to `jm_reader` for the actual parsing. This module exists to
//! maintain the `ParsedItem` / `D2IFile` types that many callers depend on,
//! plus the `parse_file` entry point.
//!
//! Parsing strategy (matching Python d2r-zero):
//! Single-pass sequential walk with forward-scan resync on alignment errors.
//! No boundary probing, no factor scoring, no dedup.

use crate::core::bitio::BitWriter;
use crate::core::encoding::encode_huffman_string;
use crate::core::ParseResult;
use crate::protocol::common::Item;
use crate::protocol::common::ItemMode;
use crate::protocol::d2i::page::split_pages;

// ── Data structures ──────────────────────────────────────────────

/// A single parsed item, tagged with page index and raw bitstream boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItem {
    pub page_index: usize,
    pub item: Item,
    /// Raw bit offset within the page's item data (bit, not byte).
    pub raw_bit_offset: usize,
    /// Total bit length consumed for this item.
    pub raw_bit_length: usize,
    /// True when this item is a socketed sub-item inside another item.
    pub is_socketed_subitem: bool,
    /// Magic prefix ID (for magic quality items)
    pub magic_prefix_id: Option<u16>,
    /// Magic suffix ID (for magic quality items)
    pub magic_suffix_id: Option<u16>,
    /// True when this item was identified conservatively and may be a false positive.
    pub is_pseudo_unverified: bool,
}

/// Complete parsed D2I file.
#[derive(Debug, Clone)]
pub struct D2IFile {
    /// Pages as parsed from the stash file.
    pub pages: Vec<crate::protocol::d2i::page::Page>,
    /// All items across all pages (main items + socketed sub-items).
    pub items: Vec<ParsedItem>,
    /// Trailing data after the last page.
    pub tail: Vec<u8>,
}

// ── Public API ──────────────────────────────────────────────────

/// Parse a complete stash file buffer into `D2IFile`.
///
/// Simple sequential reader: splits into pages, then parses each page's
/// JM item stream one item at a time.
pub fn parse_file(buffer: &[u8]) -> ParseResult<D2IFile> {
    let _t0 = std::time::Instant::now();
    let (pages, tail) = split_pages(buffer)?;
    let split_time = _t0.elapsed();
    let mut all_items = Vec::new();

    // Build StatTable once, reuse across all pages
    let _tt = std::time::Instant::now();
    let table = if crate::data::stat_loader::has_runtime_table() {
        crate::data::stat_loader::build_runtime_table()
    } else {
        crate::data::stat_cost::build_stat_table()
    };
    eprintln!("[timing] build StatTable (once) {:?}", _tt.elapsed());

    for page in &pages {
        let _tp = std::time::Instant::now();
        let items = crate::protocol::d2i::jm_reader::parse_jm_page_with_table(
            &page.data, page.index, page.is_stackable, &table);
        let page_time = _tp.elapsed();
        eprintln!("[timing] parse_file page={} is_stackable={} items={} took={:?}",
            page.index, page.is_stackable, items.len(), page_time);
        // Associate socket fillers per-page (offsets are page-local)
        let page_items = associate_socketed_items(&items);
        all_items.extend(page_items);
    }
    let total = _t0.elapsed();
    eprintln!("[timing] parse_file total pages={} items={} split={:?} parse={:?}",
        pages.len(), all_items.len(), split_time, total - split_time);
    Ok(D2IFile { pages, items: all_items, tail })
}

/// Post-process a flat list of ParsedItems to nest socket fillers inside their parent.
/// Uses the parent's `num_sockets` field to determine how many following items
/// with mode == ItemMode::Socket should be absorbed. This is more reliable than
/// range extension because D2I items may have alignment gaps (8+ bits) between
/// consecutive socket fillers.
pub fn associate_socketed_items(items: &[ParsedItem]) -> Vec<ParsedItem> {
    let mut sorted: Vec<ParsedItem> = items.to_vec();
    sorted.sort_by_key(|pi| pi.raw_bit_offset);
    let mut result: Vec<(ParsedItem, u8)> = Vec::new(); // (item, remaining_sockets)
    for pi in sorted {
        if pi.item.mode == ItemMode::Socket {
            // Try to absorb into nearest preceding parent with remaining sockets
            if let Some(idx) = result.iter().rposition(|(parent, remaining)| {
                *remaining > 0 && pi.raw_bit_offset >= parent.raw_bit_offset
            }) {
                let (parent, remaining) = &mut result[idx];
                parent.item.socketed_items.push(pi.item.clone());
                *remaining -= 1;
                continue;
            }
        }
        // Not a socket filler or no parent found — add as top-level item
        // Track remaining sockets if this item has sockets
        let ns = pi.item.num_sockets;
        result.push((pi, ns));
    }
    result.into_iter().map(|(pi, _)| pi).collect()
}

/// Parse a bare JM item stream (used by d2s diagnostic to validate Huffman).
///
/// The input must start with `b"JM"` + u16 count.
pub fn parse_item_stream_sequential(data: &[u8], page_index: usize, is_stackable: bool) -> Vec<ParsedItem> {
    if data.len() < 4 || &data[0..2] != b"JM" {
        return Vec::new();
    }
    // Create a minimal page-like buffer: 64-byte header prefix + JM data
    let mut page_buf = Vec::with_capacity(64 + data.len());
    page_buf.extend_from_slice(&[0u8; 64]);
    page_buf.extend_from_slice(data);
    crate::protocol::d2i::jm_reader::parse_jm_page(&page_buf, page_index, is_stackable)
}

// ── Feature-gated mod-stash helpers ─────────────────────────────

#[cfg(feature = "mod-stash-experimental")]
pub fn parse_file_mod_shoudao(buffer: &[u8]) -> ParseResult<D2IFile> {
    // Simplified mod stash path: same jm_reader, no special mod handling.
    // The sequential reader handles mod items because it doesn't filter
    // codes against ALL_ITEMS during parsing (only validates 3-char format).
    parse_file(buffer)
}

// ── Stackable page editing ────────────────────────────────────────

/// D2R 单格物品最大叠加数量（runes/gems/keys 等共享同一上限）。
pub const MAX_STACK: u32 = 99;

/// 在堆叠页（stackable page）上添加或减少指定 code 的物品数量。
///
/// 行为与 `protocol::d2i::legacy::item::update_stash_items` 等价，但输入输出
/// 使用新 `Page` + `ParsedItem` 类型，便于迁移 StashService / marketplace 业务。
///
/// * `delta > 0` → 增；`delta < 0` → 减；`delta == 0` → noop（返回当前 items）。
/// * `create_if_missing && delta > 0` → 找不到时从首项模板克隆创建新条目。
/// * `delta < 0` 但找不到目标 → 返回 `ItemNotFound` 错误。
///
/// 在 JM 数据中扫描指定 code 的 item，返回其在 JM payload 中的起始 byte 偏移
fn scan_item_in_jm(jm_data: &[u8], target_code: &str) -> Option<usize> {
    use crate::core::bitio::BitReader;
    use crate::core::encoding::decode_huffman_string;
    let payload = &jm_data[4..]; // skip JM header
    let mut reader = BitReader::new(payload);
    while reader.remaining_bits() >= 80 {
        let start = reader.offset();
        let _flags = reader.read_u32(32);
        let _ver = reader.read_u8(3);
        let _mode = reader.read_u8(3);
        let _loc = reader.read_u8(4);
        let _px = reader.read_u8(4);
        let _py = reader.read_u8(4);
        let _pg = reader.read_u8(3);
        let code = decode_huffman_string(&mut reader).trim().to_string();
        if code == target_code {
            return Some(start / 8);
        }
        // Skip to next item
        let _ext = reader.read_bit();
        reader.align_to_byte();
        // Realm data (conditional)
        if reader.remaining_bits() >= 32 {
            let peek = reader.peek_bits(32);
            if (peek >> 16) & 1 == 1 && reader.remaining_bits() >= 128 {
                reader.skip_bits(128);
            }
        }
        // Adv stash (conditional)
        if reader.remaining_bits() >= 136 {
            let peek = reader.peek_bits(32);
            if (peek >> 16) & 1 == 1 && reader.remaining_bits() >= 136 {
                reader.skip_bits(136);
            }
        }
    }
    None
}

/// 原位修改堆叠页物品数量——直接在 JM 字节流中改 px + realm，不重新编码。
///
/// 游戏自己的保存方式只改 2 字节（px 和 realm），
/// 保持 JM 流结构完全不变。全量重编码会因格式差异被游戏拒绝。
pub fn update_stackable_items_v2(
    stackable_page: &crate::protocol::d2i::page::Page,
    item_code: &str,
    delta: i32,
    create_if_missing: bool,
) -> Result<(Vec<ParsedItem>, Vec<u8>), String> {
    // 读出当前页上所有 items（用于返回 + 验证）
    let items: Vec<ParsedItem> = crate::protocol::d2i::jm_reader::parse_jm_page(
        &stackable_page.data,
        stackable_page.index,
        stackable_page.is_stackable,
    );

    let found_idx = items.iter().position(|pi| pi.item.code == item_code);

    if let Some(idx) = found_idx {
        let current = items[idx].item.amount;
        let new_amount = if current as i32 + delta <= 0 { 0 } else { (current as i32 + delta) as u32 };
        if new_amount > MAX_STACK {
            return Err(format!(
                "Max stack exceeded for {}. Max={}, attempted={}",
                item_code, MAX_STACK, new_amount
            ));
        }
        if new_amount == current {
            return Ok((items, stackable_page.data.clone()));
        }

        // 扫描 JM 找到 item 的实际位置
        let jm_data = &stackable_page.data[64..];
        let payload_byte_off = scan_item_in_jm(jm_data, item_code)
            .ok_or_else(|| format!("scan_item_in_jm: item {} not found in JM bitstream", item_code))?;

        let mut new_page_data = stackable_page.data.clone();
        let new_jm = &mut new_page_data[64..];

        const HDR: usize = 4; // JM header size
        const CORE_BYTES: usize = 10; // 简单堆叠项核心固定长度

        // (a) 修改 position_x (item byte 5, bits 2-5)
        let px_byte = HDR + payload_byte_off + 5;
        let new_px = (new_amount & 0x0F) as u8;
        let new_py = ((new_amount >> 4) & 0x0F) as u8;
        // px 在 byte bits 2-5, py 低 2 位在 bits 6-7
        new_jm[px_byte] = (new_jm[px_byte] & 0x03) | (new_px << 2) | ((new_py & 0x03) << 6);
        // py 高 2 位在下一个 byte 的 bits 0-1
        if px_byte + 1 < new_jm.len() {
            new_jm[px_byte + 1] = (new_jm[px_byte + 1] & 0xFC) | ((new_py >> 2) & 0x03);
        }

        // (b) 修补 realm data（仅紧凑物品，非紧凑不从 realm 读 amount）
        if items[idx].item.flags.simple_item() {
            let realm_off = HDR + payload_byte_off + CORE_BYTES + 15;
            if realm_off < new_jm.len() {
            let realm_b = new_jm[realm_off];
            let old_amt = current as u8;
            // MUL: realm = base + amount * step
            let step = [128u8, 64, 32, 16, 8, 4, 2, 1].into_iter()
                .find(|&s| (realm_b as u16) > (old_amt as u16) * (s as u16))
                .unwrap_or(1);
            let base_mul = (realm_b as i16) - (old_amt as i16) * (step as i16);
            let new_realm_val = base_mul + (new_amount as i16) * (step as i16);
            let new_realm = if base_mul >= 0 && base_mul < 256 && new_realm_val < 256 {
                // MUL 安全: 最终值不溢出 u8
                new_realm_val as u8
            } else if let Some(v) = try_div_encoding(realm_b, old_amt, new_amount) {
                v
            } else {
                // 两种公式都无法安全编码,跳过 realm 修补
                eprintln!("  ⚠ realm patch SKIPPED for {item_code} (MUL overflow, DIV lost precision)");
                new_jm[realm_off]
            };
            new_jm[realm_off] = new_realm;
            eprintln!("  realm JM[{realm_off}] = 0x{realm_b:02x} → 0x{new_realm:02x} (amt {current}→{new_amount})");
        }
        }

        // 更新返回的 items 中 amount
        let mut updated_items = items;
        updated_items[idx].item.amount = new_amount;

        eprintln!("[update_stackable_items_v2] {}: {}→{} (px byte JM[{}])",
            item_code, current, new_amount, px_byte);

        Ok((updated_items, new_page_data))
    } else if create_if_missing && delta > 0 {
        // 新增条目——fallback 到全量重编码
        let mut items = items;
        let template = items.first().ok_or("No template item available for cloning")?;
        let mut new_pi = template.clone();
        new_pi.item.code = item_code.to_string();
        new_pi.item.amount = delta as u32;
        items.push(new_pi);
        let new_item_data = encode_stackable_items(&items)?;
        let mut page_data = stackable_page.data[..64].to_vec();
        page_data.extend_from_slice(&new_item_data);
        let new_size = page_data.len() as u32;
        page_data[16..20].copy_from_slice(&new_size.to_le_bytes());
        Ok((items, page_data))
    } else if delta < 0 {
        Err(format!("Item {} not found in stash, cannot remove", item_code))
    } else {
        Ok((items, stackable_page.data.clone()))
    }
}

/// 把 stackable item 列表编码为 JM bitstream（不含 64B page header）。
///
/// 与 `protocol::d2i::legacy::item::write_stash_items` 等价：每个 item 使用
/// 固定的 32-bit flags 模板（simple_item=true，identified=true，new=true，
/// 其它字段为 0）+ v105 (3 bits "2") + 全零位置 + Huffman 4-char code +
/// 1-bit 零 sockets + 1-bit chest_stackable + 8-bit amount + align。
/// 编码单个简单物品(符文/宝石)为 JM item 比特数据(不含 JM header)。
/// 非简单物品返回 Err。
pub fn encode_item_to_jm_bits(pi: &ParsedItem) -> Result<Vec<u8>, String> {
    if !pi.item.flags.simple_item() {
        return Err("encode_item_to_jm_bits: non-simple items not supported (use warehouse deposit instead)".into());
    }
    let mut writer = BitWriter::new(128);
    encode_stackable_item(&mut writer, pi)?;
    writer.align();
    let mut out = writer.to_bytes();
    // JM reader 要求最小 80 bits(10 bytes)
    if out.len() < 10 { out.resize(10, 0); }
    Ok(out)
}
///
/// stackable page 的物品（runes/gems/keys）始终满足此约束，因此固定模板足够。
fn encode_stackable_items(items: &[ParsedItem]) -> Result<Vec<u8>, String> {
    let mut writer = BitWriter::new(items.len() * 128);

    // JM header: magic "JM" (16 bits) + item count (16 bits)
    writer.write_string("JM", 2);
    writer.write_u16(items.len() as u16, 16);

    for pi in items {
        encode_stackable_item(&mut writer, pi)?;
    }

    writer.align();
    Ok(writer.to_bytes())
}

/// DIV 编码备选: realm = (amount >> shift) + base
fn try_div_encoding(realm_b: u8, old_amt: u8, new_amount: u32) -> Option<u8> {

    for shift in 1u8..=7u8 {
        let b = (realm_b as i16) - ((old_amt as i16) >> shift);
        if b >= 0 && b < (1i16 << shift as i16) {
            // DIV 参数有效,确认新 amount 能还原
            let candidate = (new_amount >> shift) + b as u32;
            if candidate < 256 {
                // round-trip 验证
                let decoded = ((candidate as u8 - b as u8) as u32) << shift;
                if decoded == new_amount {
                    return Some(candidate as u8);
                }
            }
        }
    }
    None
}

fn encode_stackable_item(writer: &mut BitWriter, pi: &ParsedItem) -> Result<(), String> {
    encode_stackable_item_with_mode(writer, pi, 0)
}

pub fn encode_stackable_item_with_mode(writer: &mut BitWriter, pi: &ParsedItem, mode: u8) -> Result<(), String> {
    // ── 32-bit flags ──
    writer.write_bits(&[0, 0, 0, 0]);                    // b0_3
    writer.write_bit(1); writer.write_bits(&[0, 0, 0, 0, 0, 0]); // b4(identified) b5_10 (=0)
    writer.write_bit(0); writer.write_bit(0);             // b11(socketed) b12
    writer.write_bit(0); writer.write_bits(&[0, 0]);      // b13(new=0) b14_15
    writer.write_bit(0); writer.write_bit(0);             // b16(ear) b17(starter)
    writer.write_bits(&[0, 0, 0]);                        // b18_20
    writer.write_bit(1);                                  // b21(simple_item=true)
    writer.write_bit(0); writer.write_bit(1);             // b22(ethereal) b23
    writer.write_bit(0); writer.write_bit(0);             // b24(personalized) b25
    writer.write_bit(0); writer.write_bits(&[0, 0, 0, 0, 0]); // b26 b27_31

    // ── Version (5 = standard D2R, avoids unconditional realm data skip) ──
    writer.write_u16(5, 3);

    // ── Location + Position（position = amount） ──
    writer.write_u8(mode, 3);  // mode
    writer.write_u8(0, 4);   // loc
    let amt = pi.item.amount as u8;
    writer.write_u8(amt & 0x0F, 4);        // position_x = amount 低4位
    writer.write_u8((amt >> 4) & 0x0F, 4); // position_y = amount 高4位
    writer.write_u8(0, 3);  // alt_position_id

    // ── Huffman 4-char item code ──
    encode_huffman_string(writer, &pi.item.code);

    // ── compact trailer: ext_data + align + extra byte ──
    writer.write_bit(0);  // ext_data
    let rem = writer.len_bits() % 8;
    if rem != 0 {
        for _ in 0..(8 - rem) { writer.write_bit(0); }
    } else {
        writer.write_u8(0, 8);  // extra byte when already aligned
    }

    Ok(())
}

/// 编码物品(含镶嵌子物品)为 JM item 比特数据(不含 JM header)。
/// 编码完整 body + stat lists + socketed children(mode=6)。
pub fn encode_item_with_sockets(
    pi: &ParsedItem,
    table: &crate::protocol::common::StatTable,
) -> Result<Vec<u8>, String> {
    if pi.item.flags.simple_item() {
        return encode_item_to_jm_bits(pi);
    }
    // 修正父物品的 num_sockets,确保 associate_socketed_items 能吸收子物品
    let mut parent = pi.clone();
    parent.item.num_sockets = pi.item.socketed_items.len() as u8;
    parent.item.flags.raw |= 1 << 11;  // 置 socketed 标记
    
    let mut writer = BitWriter::new(512);
    encode_one_noncompact_with_mode(&mut writer, &parent, table, 0)?;
    // 编码镶嵌子物品(mode=6 = Socket)
    for child in &pi.item.socketed_items {
        let child_pi = ParsedItem {
            page_index: pi.page_index,
            item: child.clone(),
            raw_bit_offset: 0,
            raw_bit_length: 0,
            is_socketed_subitem: true,
            magic_prefix_id: None,
            magic_suffix_id: None,
            is_pseudo_unverified: false,
        };
        if child_pi.item.flags.simple_item() {
            encode_stackable_item_with_mode(&mut writer, &child_pi, 6)?;
        } else {
            encode_one_noncompact_with_mode(&mut writer, &child_pi, table, 6)?;
        }
    }
    writer.align();
    let mut out = writer.to_bytes();
    if out.len() < 10 { out.resize(10, 0); }
    Ok(out)
}

/// 编码单个非简单物品(不含镶嵌子物品)。
fn encode_one_noncompact_with_mode(
    writer: &mut BitWriter,
    pi: &ParsedItem,
    table: &crate::protocol::common::StatTable,
    mode: u8,
) -> Result<(), String> {
    let mut flags = pi.item.flags.raw;
    flags &= !(1 << 21); // clear simple_item
    flags |= 1 << 4;     // identified
    // given_runeword (bit 26): 本编码器不保存 rw_id（恒写 0），置位会导致
    // parser 按 `1u8.wrapping_shl(shift+1)` 设置 prop_lists 并读取 bonus stat 流，
    // 从而吞掉后续 socketed children 的位数据。清掉该位保持位布局一致。
    flags &= !(1 << 26);
    // flags LSB-first (BitWriter write_bit uses LSB order within byte)
    for i in 0..32 { writer.write_bit(((flags >> i) & 1) as u8); }
    writer.write_u16(5, 3); // version
    writer.write_u8(mode, 3);  // mode
    writer.write_u8(pi.item.location.as_u8(), 4);   // loc
    writer.write_u8(pi.item.x, 4); writer.write_u8(pi.item.y, 4); // px, py (from caller)
    writer.write_u8(6, 3); // pg = SharedStash(6)
    encode_huffman_string(writer, &pi.item.code);
    // Non-compact body header
    writer.write_u8(0, 3); // _soc_hint
    writer.write_u32(pi.item.id, 32);
    writer.write_u8(pi.item.item_level.min(127), 7);
    let qb = pi.item.quality.as_u8();
    writer.write_u8(qb, 4);
    writer.write_bit(0); writer.write_bit(0); // jump bits = skip
    match qb {
        7 => { writer.write_u16(pi.item.unique_id.unwrap_or(0), 12); }
        5 => { writer.write_u16(pi.item.set_id.unwrap_or(0), 12); }
        4 => {
            writer.write_u16(pi.magic_prefix_id.unwrap_or(0), 11);
            writer.write_u16(pi.magic_suffix_id.unwrap_or(0), 11);
        }
        6 | 8 => { writer.write_u16(0, 16); for _ in 0..6 { writer.write_bit(0); } }
        1 | 3 => { writer.write_u8(0, 3); }
        _ => {}
    }
    if (flags >> 26) & 1 == 1 {
        writer.write_u16(0, 12);
        // shift=7 → parse 端 `1u8.wrapping_shl(7+1)` 溢出为 0 → prop_lists=0，
        // 避免 parser 误读 bonus stat 流（否则会吞掉后续 socketed children 数据）。
        writer.write_u8(7, 4);
    }
    if (flags >> 16) & 1 == 1 { writer.write_u8(0, 10); writer.write_u8(0, 8); }
    else if (flags >> 24) & 1 == 1 { writer.write_u8(0, 8); }
    writer.write_bit(0); // realm
    // Type-specific
    let (ca, cw, cs) = crate::protocol::d2i::legacy::complete_header::lookup_item_category(&pi.item.code);
    if ca || cs {
        let sb31 = table.get(31).save_bits.max(1) as u16;
        writer.write_u16(0, sb31 as u8);
        let sb73 = table.get(73).save_bits.max(1);
        writer.write_u16(0, sb73);
    } else if cw {
        let sb73 = table.get(73).save_bits.max(1);
        writer.write_u16(0, sb73);
    } else if (flags >> 28) & 1 == 1 { writer.write_u8(0, 9); }
    if (flags >> 11) & 1 == 1 { writer.write_u8(pi.item.num_sockets.min(15), 4); }
    if qb == 5 { writer.write_u8(0, 5); }
    // Stat lists — 写入空列表(仅终止符),避免位偏移导致乱码。
    // 游戏下次保存时会根据 item 数据重新编码正确的 stat。
    writer.write_u16(0x1FF, 9);
    writer.align();
    // Post-body realm (bit 16 = 0 → skip)
    writer.write_u32(0, 32);
    writer.align();
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_file(&[]);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.items.len(), 0);
        assert_eq!(file.pages.len(), 0);
    }

    #[test]
    fn test_parse_minimal() {
        // Just the 4-byte d2i magic with no pages
        let buf = vec![0x55, 0xAA, 0x55, 0xAA, 0x00, 0x00, 0x00, 0x00];
        let result = parse_file(&buf);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.items.len(), 0);
    }
}
