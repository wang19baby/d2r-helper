use serde::{Deserialize, Serialize};

use super::bit_reader::BitReader;
use super::bit_writer::BitWriter;
#[allow(unused_imports)]
use super::game_items::ALL_ITEMS;
use super::huffman::{decode_huffman_string, encode_huffman_string};
#[allow(unused_imports)]
use super::magical_props::MAGICAL_PROPS;
use super::page::{find_stackable_page, Page};

/// Maximum stack size for items in the shared stash
const MAX_STACK: u8 = 99;
use crate::data::stat_cost::build_stat_table;

/// Game version constant used by D2R
const GAME_VERSION: u8 = 105;

/// Re-export from complete_header module for backward compatibility
pub use super::complete_header::lookup_item_category;

/// Item type category from the game
#[repr(u8)]
#[allow(dead_code)]
enum ItemCategory {
    Armor = 1,
    Shield = 2,
    Weapon = 3,
    Other = 4,
}

/// An item parsed from the shared stash stackable page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashItem {
    /// Item type code (e.g. "r01", "gcv", "rvs")
    pub item_type: String,
    /// Display name
    pub name: Option<String>,
    /// Quantity in the stash
    pub amount: u32,
    /// Item quality level (1=Low,2=Normal,3=Superior,4=Magic,5=Set,6=Rare,7=Unique,8=Crafted)
    pub quality: Option<u8>,
    /// Whether this is a simple (non-magic) item
    pub simple_item: bool,
    /// Whether this item is identified
    pub identified: bool,
    /// Whether this item is socketed
    pub socketed: bool,
    /// Whether this item is ethereal
    pub ethereal: bool,
    /// Raw position data
    pub position_x: u8,
    pub position_y: u8,
    pub location_id: u8,
    pub alt_position_id: u8,
    /// Inventory width (number of grid cells)
    pub inv_width: u8,
    /// Inventory height (number of grid cells)
    pub inv_height: u8,
    /// Bit offset of this item's raw data in the item bitstream (relative to page data start at byte 64)
    pub raw_bit_offset: usize,
    /// Total bits this item occupies in the raw item bitstream
    pub raw_bit_length: usize,
    /// Unknown data preserved from the original parsing
    pub unknown_data: Vec<u8>,
}

/// Set of item type codes that are supported (known by the inventory system)
pub const SUPPORTED_ITEM_CODES: &[&str] = &[
    "r01", "r02", "r03", "r04", "r05", "r06", "r07", "r08", "r09", "r10",
    "r11", "r12", "r13", "r14", "r15", "r16", "r17", "r18", "r19", "r20",
    "r21", "r22", "r23", "r24", "r25", "r26", "r27", "r28", "r29", "r30",
    "r31", "r32", "r33",
    "gcv","gcw","gcg","gcr","gcb","gcy","skc",
    "gfv","gfw","gfg","gfr","gfb","gfy","skf",
    "gsv","gsw","gsg","gsr","gsb","gsy","sku",
    "gzv","glw","glg","glr","glb","gly","skl",
    "gpv","gpw","gpg","gpr","gpb","gpy","skz",
    "rvs","rvl",
    "pk1","pk2","pk3",
    "toa",
    "tes","ceh","bet","fed",
    "xa1","xa2","xa3","xa4","xa5",
];

/// Read all stackable items from a stash file's stackable page.
/// Returns ALL items (both simple and non-simple), with raw bit offset tracking
/// for faithful binary round-trip.
pub fn read_stash_items(pages: &[Page]) -> Result<Vec<StashItem>, String> {
    let stack_page = find_stackable_page(pages)
        .ok_or_else(|| "No stackable page found".to_string())?;
    read_stash_items_from_page(stack_page)
}

#[allow(dead_code)]
fn scan_for_next_item(reader: &mut BitReader, scan_start: usize) {
    // Header signature: bits 0-3=0000, bits 5-10=000100
    //  ⇒ byte0 = 0x00|0x10 (lower nibble=0), byte1 & 0x07 == 0x01 (bit0=1,bit1=0,bit2=0)
    let scan_limit = reader.len_bits().min(scan_start + 2048);
    let mut scan_off = scan_start;
    while scan_off + 24 <= scan_limit {
        reader.seek(scan_off);
        let b0 = reader.read_u16(8) as u8;
        let b1 = reader.read_u16(8) as u8;
        if (b0 & 0x0F) == 0x00 && (b1 & 0x07) == 0x01 {
            reader.seek(scan_off);
            return;
        }
        scan_off += 8;
    }
}

/// Read items from a specific page (not just the stackable page).
/// Each page in the .d2i file has its own item bitstream starting with "JM" magic.
/// Items that fail to parse are skipped; only items with known codes are returned.
/// When an item's parsed code doesn't match the game tables, the reader scans
/// forward for the next item header to resync the bitstream.
pub fn read_stash_items_from_page(page: &Page) -> Result<Vec<StashItem>, String> {
    let item_data = &page.data[64..];
    let mut reader = BitReader::new(item_data);

    let magic = reader.read_string(2);
    if magic != "JM" {
        // No JM header — treat as empty page (page data is zeroed/unused)
        return Ok(Vec::new());
    }
    let count = reader.read_u16(16) as usize;
    let mut items = Vec::with_capacity(count);

    for _ in 0..count {
        let start_bit = reader.offset();
        if start_bit >= reader.len_bits() {
            break;
        }
        if let Ok(mut item) = read_single_item(&mut reader, 0, page.is_stackable) {
            let end_bit = reader.offset();
            item.raw_bit_offset = start_bit;
            item.raw_bit_length = end_bit - start_bit;
            if item.amount > 0 {
                let code = item.item_type.trim();
                if !code.is_empty() && !code.is_empty() {
                    let in_table = crate::protocol::d2i::legacy::game_items::ALL_ITEMS
                        .iter().any(|(c, _, _, _, _)| *c == code)
                        || crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP
                            .iter().any(|(c, _, _, _)| *c == code);
                    if in_table {
                        items.push(item);
                    }
                }
            }
        }
    }

    Ok(items)
}

/// Read items from ALL pages in the stash file.
/// Errors from any page propagate immediately (fail-fast).
pub fn read_all_stash_items(pages: &[Page]) -> Result<Vec<(usize, Vec<StashItem>)>, String> {
    let mut result = Vec::with_capacity(pages.len());
    for page in pages {
        let items = read_stash_items_from_page(page)?;
        result.push((page.index, items));
    }
    Ok(result)
}

/// Extract the raw bitstream bytes for a StashItem from the stash page data.
/// Uses the item's raw_bit_offset and raw_bit_length.
pub fn extract_raw_item_bits(item_data: &[u8], item: &StashItem) -> Vec<u8> {
    let start_byte = item.raw_bit_offset / 8;
    let end_bit = item.raw_bit_offset + item.raw_bit_length;
    let end_byte = end_bit.div_ceil(8);
    if start_byte < end_byte && end_byte <= item_data.len() {
        item_data[start_byte..end_byte].to_vec()
    } else {
        Vec::new()
    }
}

/// Look up inventory size for a given item code.
/// Returns (width, height) in grid cells. Defaults to (1, 1) if not found.
/// Priority: runtime TXT data ?hardcoded ITEM_INVENTORY_SIZES ?(1, 1)
pub fn get_item_inventory_size(code: &str) -> (u8, u8) {
    // Runtime (TXT-loaded) data first ?more accurate than hardcoded
    crate::protocol::d2i::legacy::game_data_loader::get_inventory_size(code)
}

/// Read a single item from the bitstream.
/// Handles both simple_item=true (stackable: runes, gems, potions)
/// and simple_item=false (equipment, charms) layout differences.
/// `on_stackable_page`: if true, the chest-stackable trailer (1+8 bits)
/// is always present at the end, even for non-simple items.
/// On non-stackable pages, only simple items have the trailer.
fn read_single_item(reader: &mut BitReader, _prop_budget: usize, on_stackable_page: bool) -> Result<StashItem, String> {
    // ?
    // SECTION 1: Header flags (32 bits total)
    // ?
    let _b0_3 = reader.read_bit_array(4);
    let identified_bit = reader.read_bit();
    let _b5_10 = reader.read_bit_array(6);
    let socketed_bit = reader.read_bit();
    let _b12 = reader.read_bit_array(1);
    let _new = reader.read_bit();
    let _b14_15 = reader.read_bit_array(2);
    let _is_ear = reader.read_bit();
    let _starter_item = reader.read_bit();
    let _b18_20 = reader.read_bit_array(3);
    let simple_item_bit = reader.read_bit();
    let ethereal_bit = reader.read_bit();
    let _b23 = reader.read_bit_array(1);
    let personalized_bit = reader.read_bit();
    let _b25 = reader.read_bit_array(1);
    let given_runeword_bit = reader.read_bit();
    let _b27_31 = reader.read_bit_array(5);

    let simple_item = simple_item_bit == 1;
    let identified = identified_bit == 1;
    let socketed = socketed_bit == 1;
    let personalized = personalized_bit == 1;
    let given_runeword = given_runeword_bit == 1;
    let is_ear = _is_ear == 1;

    // ?
    // SECTION 2: Version + Location + Item Type (fixed for all items)
    // ?
    let _version = reader.read_u16(3); // "2" for v105

    let _location_id = reader.read_u8(3);
    let _equipped_id = reader.read_u8(4);
    let position_x = reader.read_u8(4);
    let position_y = reader.read_u8(4);
    let _alt_position_id = reader.read_u8(3);

    // Item type (Huffman encoded 4 chars for v97+)
    // IMPORTANT: trim trailing spaces — ALL_ITEMS and other tables use trimmed codes
    let item_type = decode_huffman_string(reader).trim().to_string();

    // Look up inventory size
    let (inv_width, inv_height) = get_item_inventory_size(&item_type);

    // nr_of_items_in_sockets: 1 bit for simple items, 3 bits for non-simple
    let _nr_of_sockets_hint = if simple_item { reader.read_u8(1) } else { reader.read_u8(3) };

    // ?
    // SECTION 3: Non-simple item body (SKIPPED for simple items)
    // ?
    let item_quality: Option<u8> = if !simple_item {
        let q = skip_non_simple_item_body_inner(reader, &item_type, identified, socketed,
                                          personalized, given_runeword, is_ear,
                                          GAME_VERSION, false, &build_stat_table())?;
        Some(q)
    } else {
        None
    };

    // ?
    // SECTION 3b: Chronicle (version 105, only for unidentified unique/set items)
    // ?
    // The game stores the monster kill that dropped this item.
    // Condition: version is 105, item is identified OR not (unique/set)
    // If identified == false AND quality is Unique(7)/Set(5): chronicle = 16+32+4 = 52 bits
    // Otherwise: chronicle is skipped (0 bits)
    if GAME_VERSION == 105 && !identified
        && let Some(q) = item_quality
            && (q == 5 || q == 7) {
                // monsterId (16 bits) + timestamp (32 bits) + padding (4 bits)
                reader.skip_bits(52);
            }

    // ? 仙道轮回 mod 扩展: 0x1FF 后 8b metadata (default 0x00)
    // 所有 non-stackable、non-simple 装备都有
    // [BAC] d2rr_toolkit 在此之后仅有 tracking_trailer(48b if bit28)+byte_align。
    // skip_non_simple_item_body_inner 已在 property list 后读了 8b mod_metadata，
    // 此处是第二个 8b（共 16b）。两边路径（old read_single_item vs new parser.rs）
    // 消费不同位数，需用真实 fixture 对账确认。d2rr_toolkit 无 chronicle 也无第二个 8b。
    // 暂不删除——但若对齐出问题，优先怀疑这里。
    if !on_stackable_page && !simple_item {
        reader.skip_bits(8);
    }

    // ?
    // SECTION 4: Chest-stackable trailer (version 105)
    // Only present for items on the stackable page (the stash's "stackable"
    // tab, used for runes/gems/essences/charms). Equipment items on
    // non-stackable pages do NOT have this trailer — reading it would
    // consume bits from padding or the next item, producing wildly wrong
    // "amount" values like 255 or 325.
    // ?
    let has_trailer = on_stackable_page || simple_item;
    let amount = if has_trailer {
        let chest_stackable = reader.read_bit();
        if chest_stackable == 1 {
            reader.read_u8(8) as u32
        } else {
            1
        }
    } else {
        1
    };

    // Align to byte boundary
    reader.align();

    Ok(StashItem {
        item_type: item_type.trim().to_string(),
        name: None,
        amount,
        quality: item_quality,
        simple_item,
        identified,
        socketed,
        ethereal: ethereal_bit == 1,
        position_x,
        position_y,
        location_id: _location_id,
        alt_position_id: _alt_position_id,
        inv_width,
        inv_height,
        raw_bit_offset: 0,
        raw_bit_length: 0,
        unknown_data: Vec::new(),
    })
}

/// Re-export from complete_header module for backward compatibility
pub use super::complete_header::skip_non_simple_complete_header;
pub(super) use super::complete_header::skip_non_simple_item_body_inner;

/// Reassemble the item bitstream from raw item bit sections.
/// This allows adding/removing items without needing full item re-serialization.
/// `raw_item_sections`: Vec of (raw_bytes_bit_offset, raw_bit_length, raw_bytes) for each item.
pub fn reassemble_item_stream(raw_item_sections: &[(usize, usize, Vec<u8>)], item_data: &[u8]) -> Vec<u8> {
    use super::bit_writer::BitWriter;
    let mut writer = BitWriter::new(item_data.len() + 4096);

    // Write header: "JM" + count
    writer.write_string("JM", 2);
    writer.write_u16(raw_item_sections.len() as u16, 16);

    // Write each item's raw bits
    for (_start_bit, bit_len, raw_bytes) in raw_item_sections {
        if *bit_len == 0 || raw_bytes.is_empty() {
            continue;
        }
        // Convert bytes to bits using a temp BitReader and copy
        let mut temp_reader = super::bit_reader::BitReader::new(raw_bytes);
        for _ in 0..*bit_len {
            writer.write_bit(temp_reader.read_bit());
        }
    }

    writer.align();
    writer.to_bytes()
}

/// Copy a range of bits from a byte slice to a BitWriter
#[allow(dead_code)]
fn copy_bits_from_slice(writer: &mut BitWriter, data: &[u8], bit_offset: usize, bit_count: usize) {
    let mut reader = super::bit_reader::BitReader::new(data);
    reader.seek(bit_offset);
    for _ in 0..bit_count {
        if reader.is_empty() { break; }
        writer.write_bit(reader.read_bit());
    }
}

/// Write items back to a stackable page's item data buffer.
/// NOTE: Only works correctly for simple (stackable) items.
/// For non-simple items, use reassemble_item_stream instead.
pub fn write_stash_items(items: &[StashItem]) -> Result<Vec<u8>, String> {
    let mut writer = BitWriter::new(items.len() * 128);

    //  Header 
    writer.write_string("JM", 2);
    writer.write_u16(items.len() as u16, 16);

    for item in items {
        write_single_item(&mut writer, item)?;
    }

    writer.align();
    Ok(writer.to_bytes())
}

/// Write a single item to the bitstream
fn write_single_item(writer: &mut BitWriter, item: &StashItem) -> Result<(), String> {
    //  Flags (preserved defaults) 
    writer.write_bits(&[0, 0, 0, 0]); // b0_3
    writer.write_bit(1); // identified
    writer.write_bits(&[0, 0, 0, 0, 0, 1]); // b5_10
    writer.write_bit(0); // socketed
    writer.write_bit(0); // b12
    writer.write_bit(1); // new
    writer.write_bits(&[0, 0]); // b14_15
    writer.write_bit(0); // is_ear
    writer.write_bit(0); // starter_item
    writer.write_bits(&[0, 0, 0]); // b18_20
    writer.write_bit(1); // simple_item = true (stackables are always simple)
    writer.write_bit(0); // ethereal
    writer.write_bit(1); // b23
    writer.write_bit(0); // personalized
    writer.write_bit(0); // b25
    writer.write_bit(0); // given_runeword
    writer.write_bits(&[0, 0, 0, 0, 0]); // b27_31

    //  Version (105 = "2" in 3 bits) 
    writer.write_u16(2, 3);

    //  Location 
    writer.write_u8(0, 3); // location_id
    writer.write_u8(0, 4); // equipped_id
    writer.write_u8(item.position_x, 4);
    writer.write_u8(item.position_y, 4);
    writer.write_u8(0, 3); // alt_position_id

    //  Item type (Huffman encoded) 
    encode_huffman_string(writer, &item.item_type);

    //  nr_of_items_in_sockets (★ BUG FIX: simple_item→1 bit, non-simple→3 bits)
    //  Writer always sets simple_item=true for stackables, so 1 bit matches reader.
    writer.write_u8(0, 1);

    //  Chest-stackable trailer 
    writer.write_bit(1); // chest_stackable = true
    writer.write_u8(item.amount as u8, 8); // amount

    writer.align();

    Ok(())
}

/// Update an existing stash file: add or remove items
/// Returns (items, updated_page_data)
pub fn update_stash_items(
    stackable_page: &Page,
    item_code: &str,
    delta: i32,
    create_if_missing: bool,
) -> Result<(Vec<StashItem>, Vec<u8>), String> {
    let mut items = read_stash_items(std::slice::from_ref(stackable_page))?;

    let found = items.iter_mut().find(|i| i.item_type == item_code);

    if let Some(item) = found {
        let new_amount = item.amount as i32 + delta;
        if new_amount <= 0 {
            // Remove the item
            items.retain(|i| i.item_type != item_code);
        } else if new_amount > MAX_STACK as i32 {
            return Err(format!(
                "Max stack exceeded for {}. Max={}, attempted={}",
                item_code, MAX_STACK, new_amount
            ));
        } else {
            item.amount = new_amount as u32;
        }
    } else if create_if_missing && delta > 0 {
        // Create new item by cloning from template
        let template = items.first().ok_or("No template item available for cloning")?;
        let new_item = StashItem {
            item_type: item_code.to_string(),
            amount: delta as u32,
            ..template.clone()
        };
        items.push(new_item);
    } else if delta < 0 {
        return Err(format!(
            "Item {} not found in stash, cannot remove",
            item_code
        ));
    }

    // Re-write the item data
    let new_item_data = write_stash_items(&items)?;

    // Build the updated page data (64-byte header + new item data)
    let mut page_data = stackable_page.data[..64].to_vec();
    page_data.extend_from_slice(&new_item_data);

    // Update the page size in the header
    let new_size = page_data.len() as u32;
    page_data[16..20].copy_from_slice(&new_size.to_le_bytes());

    Ok((items, page_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_item() {
        // Create a minimal bitstream for a simple stackable item
        let mut writer = BitWriter::new(256);
        writer.write_string("JM", 2);
        writer.write_u16(1, 16); // 1 item

        // Item flags
        writer.write_bits(&[0,0,0,0]); writer.write_bit(1); // identified
        writer.write_bits(&[0,0,0,0,0,1]);
        writer.write_bit(0); writer.write_u8(0, 1); writer.write_bit(1);
        writer.write_bits(&[0,0]); writer.write_bit(0); writer.write_bit(0);
        writer.write_bits(&[0,0,0]); writer.write_bit(1); // simple
        writer.write_bit(0); writer.write_bit(1); writer.write_bit(0);
        writer.write_bit(0); writer.write_bits(&[0,0,0,0,0]);
        writer.write_u16(2, 3); // version 105
        writer.write_u8(0, 3); writer.write_u8(0, 4); // loc
        writer.write_u8(0, 4); writer.write_u8(0, 4); // pos
        writer.write_u8(0, 3);

        // Huffman "r01"
        for c in "r01".chars() {
            super::super::huffman::encode_huffman_char(&mut writer, c);
        }
        // Pad to 4 chars
        super::super::huffman::encode_huffman_char(&mut writer, ' ');
        writer.write_u8(0, 3); // nr_of_sockets

        // Chest-stackable
        writer.write_bit(1);
        writer.write_u8(5, 8); // amount = 5
        writer.align();

        let data = writer.to_bytes();
        let mut reader = BitReader::new(&data);
        let magic = reader.read_string(2);
        assert_eq!(magic, "JM");
        let count = reader.read_u16(16);
        assert_eq!(count, 1);
    }
}

#[cfg(test)]
mod item_passthrough_tests {
    use super::*;

    fn find_test_stash() -> Option<String> {
        let candidates = [
            "D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/src-tauri/tests/fixtures/ModernSharedStashSoftCoreV2.d2i",
            "D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/src-tauri/tests/fixtures/user_stash.d2i",
            "D:/work_space/personal_workspace/d2r/d2r-marketplace/tools/d2r_parser/test.d2i",
        ];
        for p in &candidates {
            if std::path::Path::new(p).exists() {
                return Some(p.to_string());
            }
        }
        None
    }

    #[test]
    fn test_raw_item_passthrough() {
        let stash_path = match find_test_stash() {
            Some(p) => p,
            None => { eprintln!("SKIP: no test .d2i"); return; }
        };
        let data = std::fs::read(&stash_path).expect("read stash");
        let pages = super::super::page::split_legacy_d2i_pages(&data).expect("parse pages");
        let stack_page = super::super::page::find_stackable_page(&pages.pages).expect("stack page");
        let item_data = &stack_page.data[64..];
        let items = read_stash_items(&pages.pages).expect("read items");
        assert!(!items.is_empty(), "no items found");

        for (i, item) in items.iter().enumerate() {
            assert!(item.raw_bit_length > 0, "item {} raw_bit_length=0", i);
        }

        let sections: Vec<_> = items.iter().map(|item| {
            (item.raw_bit_offset, item.raw_bit_length, extract_raw_item_bits(item_data, item))
        }).collect();

        let rebuilt = reassemble_item_stream(&sections, item_data);
        assert_eq!(&rebuilt[0..2], b"JM");
        let count = u16::from_le_bytes([rebuilt[2], rebuilt[3]]);
        assert_eq!(count as usize, items.len(), "count mismatch");
        eprintln!("?Passthrough: {} items, {} -> {} bytes", items.len(), item_data.len(), rebuilt.len());
    }

    #[test]
    fn test_extract_one_item() {
        let stash_path = match find_test_stash() {
            Some(p) => p,
            None => { eprintln!("SKIP: no test .d2i"); return; }
        };
        let data = std::fs::read(&stash_path).expect("read stash");
        let pages = super::super::page::split_legacy_d2i_pages(&data).expect("parse pages");
        let stack_page = super::super::page::find_stackable_page(&pages.pages).expect("stack page");
        let item_data = &stack_page.data[64..];
        let items = read_stash_items(&pages.pages).expect("read items");
        if items.len() < 2 { eprintln!("SKIP: need 2+ items"); return; }

        let remaining: Vec<_> = items[1..].iter().map(|item| {
            (item.raw_bit_offset, item.raw_bit_length, extract_raw_item_bits(item_data, item))
        }).collect();

        let rebuilt = reassemble_item_stream(&remaining, item_data);
        let count = u16::from_le_bytes([rebuilt[2], rebuilt[3]]);
        assert_eq!(count as usize, items.len() - 1);
        eprintln!("?Extract: {} -> {} items", items.len(), count);
    }
}
#[cfg(test)]
#[cfg(feature = "local-debug-tests")]
mod page3_debug {
    use super::*;
    use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;

    #[test]
    fn test_page3_bitmap() {
        use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[3];
        let raw = &page.data[64..];

        eprintln!("======================================");
        eprintln!(" Page[3] BITMAP — amulet (Rare)");
        eprintln!("======================================");

        // Step through EVERY field with absolute offsets
        let mut r = BitReader::new(raw);
        let orig = |r: &BitReader| r.offset();

        // ====== "JM" + count ======
        let _jm = r.read_string(2);
        eprintln!("  {:-4}..{:<4}  JM magic (= 0x4A 0x4D)", orig(&r)-16, orig(&r));

        let _cnt = r.read_u16(16);
        eprintln!("  {:-4}..{:<4}  item_count={}", orig(&r)-16, orig(&r), _cnt);

        // ====== Item 0 header 32 bits ======
        let h0 = orig(&r);
        let _b0_3 = r.read_bit_array(4);
        eprintln!("  {:-4}..{:<4}  hdr_b0_3={:?}", h0, orig(&r), _b0_3);
        let h1 = orig(&r);
        let _id = r.read_bit();
        eprintln!("  {:-4}..{:<4}  identified={}", h1, orig(&r), _id);
        let h2 = orig(&r);
        let _b5_10 = r.read_bit_array(6);
        eprintln!("  {:-4}..{:<4}  hdr_b5_10={:?}", h2, orig(&r), _b5_10);
        let h3 = orig(&r);
        let _soc = r.read_bit();
        eprintln!("  {:-4}..{:<4}  socketed={}", h3, orig(&r), _soc);
        let h4 = orig(&r);
        let _b12 = r.read_bit();
        eprintln!("  {:-4}..{:<4}  hdr_b12={}", h4, orig(&r), _b12);
        let h5 = orig(&r);
        let _new = r.read_bit();
        eprintln!("  {:-4}..{:<4}  new_item={}", h5, orig(&r), _new);
        let h6 = orig(&r);
        let _b14_15 = r.read_bit_array(2);
        eprintln!("  {:-4}..{:<4}  hdr_b14_15={:?}", h6, orig(&r), _b14_15);
        let h7 = orig(&r);
        let _ear = r.read_bit();
        eprintln!("  {:-4}..{:<4}  is_ear={}", h7, orig(&r), _ear);
        let h8 = orig(&r);
        let _st = r.read_bit();
        eprintln!("  {:-4}..{:<4}  starter_item={}", h8, orig(&r), _st);
        let h9 = orig(&r);
        let _b18_20 = r.read_bit_array(3);
        eprintln!("  {:-4}..{:<4}  hdr_b18_20={:?}", h9, orig(&r), _b18_20);
        let h10 = orig(&r);
        let _simp = r.read_bit();
        eprintln!("  {:-4}..{:<4}  simple_item={}", h10, orig(&r), _simp);
        let h11 = orig(&r);
        let _eth = r.read_bit();
        eprintln!("  {:-4}..{:<4}  ethereal={}", h11, orig(&r), _eth);
        let h12 = orig(&r);
        let _b23 = r.read_bit();
        eprintln!("  {:-4}..{:<4}  hdr_b23={}", h12, orig(&r), _b23);
        let h13 = orig(&r);
        let _pers = r.read_bit();
        eprintln!("  {:-4}..{:<4}  personalized={}", h13, orig(&r), _pers);
        let h14 = orig(&r);
        let _b25 = r.read_bit();
        eprintln!("  {:-4}..{:<4}  hdr_b25={}", h14, orig(&r), _b25);
        let h15 = orig(&r);
        let _rw = r.read_bit();
        eprintln!("  {:-4}..{:<4}  given_runeword={}", h15, orig(&r), _rw);
        let h16 = orig(&r);
        let _b27_31 = r.read_bit_array(5);
        eprintln!("  {:-4}..{:<4}  hdr_b27_31={:?}  ← 32bit header end", h16, orig(&r), _b27_31);

        // ====== Version + Location ======
        let v0 = orig(&r);
        let _ver = r.read_u16(3);
        eprintln!("  {:-4}..{:<4}  version={}", v0, orig(&r), _ver);
        let v1 = orig(&r);
        let _loc = r.read_u8(3);
        eprintln!("  {:-4}..{:<4}  location_id={} (0=storage,1=equip,...)", v1, orig(&r), _loc);
        let v2 = orig(&r);
        let _eq = r.read_u8(4);
        eprintln!("  {:-4}..{:<4}  equipped_id={}", v2, orig(&r), _eq);
        let v3 = orig(&r);
        let _px = r.read_u8(4);
        eprintln!("  {:-4}..{:<4}  position_x={}", v3, orig(&r), _px);
        let v4 = orig(&r);
        let _py = r.read_u8(4);
        eprintln!("  {:-4}..{:<4}  position_y={}", v4, orig(&r), _py);
        let v5 = orig(&r);
        let _alt = r.read_u8(3);
        eprintln!("  {:-4}..{:<4}  alt_position_id={}", v5, orig(&r), _alt);

        // ====== Huffman item type code ======
        let ht_start = orig(&r);
        let code = crate::protocol::d2i::legacy::huffman::decode_huffman_string(&mut r);
        eprintln!("  {:-4}..{:<4}  huffman_type=\"{}\" ({}/{}/{}/{})",
            ht_start, orig(&r), code.trim(),
            code.chars().next().map(|c| format!("'{}'={}b", c,
                crate::protocol::d2i::legacy::huffman::HUFFMAN_LOOKUP.iter()
                    .find(|(ch,_,_)| *ch==c).map(|(_,_,b)|*b).unwrap_or(0)
                )
            ).unwrap_or_default(),
            code.chars().nth(1).map(|c| format!("'{}'={}b", c,
                crate::protocol::d2i::legacy::huffman::HUFFMAN_LOOKUP.iter()
                    .find(|(ch,_,_)| *ch==c).map(|(_,_,b)|*b).unwrap_or(0)
                )
            ).unwrap_or_default(),
            code.chars().nth(2).map(|c| format!("'{}'={}b", c,
                crate::protocol::d2i::legacy::huffman::HUFFMAN_LOOKUP.iter()
                    .find(|(ch,_,_)| *ch==c).map(|(_,_,b)|*b).unwrap_or(0)
                )
            ).unwrap_or_default(),
            code.chars().nth(3).map(|c| format!("'{}'={}b", c,
                crate::protocol::d2i::legacy::huffman::HUFFMAN_LOOKUP.iter()
                    .find(|(ch,_,_)| *ch==c).map(|(_,_,b)|*b).unwrap_or(0)
                )
            ).unwrap_or_default(),
        );

        // ====== nr_of_sockets_hint ======
        let s0 = orig(&r);
        let _soc_hint = r.read_u8(3);
        eprintln!("  {:-4}..{:<4}  nr_of_sockets_hint={} (non-simple→3bit)", s0, orig(&r), _soc_hint);

        // ====== BODY: item_id ======
        let b0 = orig(&r);
        let _item_id = r.read_u32(32);
        eprintln!("  {:-4}..{:<4}  body: item_id={} ({:#010x})", b0, orig(&r), _item_id, _item_id);

        // ====== BODY: level ======
        let b1 = orig(&r);
        let _lvl = r.read_u8(7);
        eprintln!("  {:-4}..{:<4}  body: ilvl={}", b1, orig(&r), _lvl);

        // ====== BODY: quality ======
        let b2 = orig(&r);
        let _q = r.read_u8(4);
        let qnames = ["","Low","Normal","Superior","Magic","Set","Rare","Unique","Crafted"];
        let qn = if (_q as usize) < qnames.len() { qnames[_q as usize] } else {"?"};
        eprintln!("  {:-4}..{:<4}  body: quality={} ({})", b2, orig(&r), _q, qn);

        // ====== BODY: multi_pic + picture_id ======
        let b3 = orig(&r);
        let _mp = r.read_bit();
        eprintln!("  {:-4}..{:<4}  body: multi_pic={}", b3, orig(&r), _mp);
        if _mp == 1 {
            let b3a = orig(&r);
            let _pid = r.read_u8(3);
            eprintln!("  {:-4}..{:<4}  body:   picture_id={}", b3a, orig(&r), _pid);
        }

        // ====== BODY: class_specific ======
        let b4 = orig(&r);
        let _cs = r.read_bit();
        eprintln!("  {:-4}..{:<4}  body: class_specific={}", b4, orig(&r), _cs);
        if _cs == 1 {
            let b4a = orig(&r);
            let _aa = r.read_u16(11);
            eprintln!("  {:-4}..{:<4}  body:   auto_affix_id={}", b4a, orig(&r), _aa);
        }

        // ====== BODY: Rare quality ======
        let b5 = orig(&r);
        let _rn1 = r.read_u8(8);
        let _rn2 = r.read_u8(8);
        eprintln!("  {:-4}..{:<4}  body: rare_name1={} rare_name2={}",
            b5, orig(&r), _rn1, _rn2);

        for ai in 0..6 {
            let bf = orig(&r);
            let f = r.read_bit();
            if f == 1 {
                let _bfa = orig(&r);
                let _aff = r.read_u16(11);
                let aff_type = if _aff < 728 { "prefix" } else { "suffix" };
                eprintln!("  {:-4}..{:<4}  body:   affix[{}]=1 id={} ({}, raw={})",
                    bf, orig(&r), ai, _aff, aff_type, _aff);
            } else {
                eprintln!("  {:-4}..{:<4}  body:   affix[{}]=0", bf, orig(&r), ai);
            }
        }

        // ====== BODY: given_runeword ======
        let b6 = orig(&r);
        // already read as header flag - skip
        eprintln!("  {:-4}..{:<4}  body: given_runeword (0 bits, flag=0)", b6, orig(&r));

        // ====== BODY: personalized ======
        let b7 = orig(&r);
        eprintln!("  {:-4}..{:<4}  body: personalized (0 bits, flag=0)", b7, orig(&r));

        // ====== BODY: defense ======
        // lookup shows amu is not armor/shield
        eprintln!("  {:-4}..{:<4}  body: defense (0 bits, amu≠armor/shield)", orig(&r), orig(&r));

        // ====== BODY: durability ======
        eprintln!("  {:-4}..{:<4}  body: durability (0 bits, amu≠armor/weapon/shield)", orig(&r), orig(&r));

        // ====== BODY: v105 block ======
        let v105_start = orig(&r);
        r.skip_bits(2); // v105 header
        eprintln!("  {:-4}..{:<4}  body: v105_header (2 bits)", v105_start, orig(&r));

        // ====== BODY: socketed ======
        eprintln!("  {:-4}..{:<4}  body: socketed (0 bits, flag=0)", orig(&r), orig(&r));

        // ====== BODY: plist_flag ======
        eprintln!("  {:-4}..{:<4}  body: plist_flag (0 bits, quality≠Set)", orig(&r), orig(&r));

        // ====== MAGIC PROPERTIES ======
        let mp_start = orig(&r);
        eprintln!("\n  === MAGIC PROPERTIES START at raw_offset={} (item_offset={}) ===",
            mp_start, mp_start - 32);

        loop {
            if r.offset() + 9 > r.len_bits() { break; }
            let sid = r.read_u16(9);
            if sid == 0x1FF {
                eprintln!("  {:-4}..{:<4}  magic: 0x1FF ≡ TERMINATOR  ← magic properties end", r.offset()-9, r.offset());
                break;
            }

            let sid_u = sid as usize;
            let mp_field_start = r.offset() - 9;

            if sid_u >= MAGICAL_PROPS.len() {
                eprintln!("  {:-4}..{:<4}  magic: UNKNOWN sid={} (skip 8 bits)", mp_field_start, r.offset(), sid);
                r.skip_bits(8);
                continue;
            }

            let p = &MAGICAL_PROPS[sid_u];
            let sp = p.save_param_bits;
            let sb = p.save_bits;
            let sa = p.save_add;
            let np = p.num_sub_props.max(1) as usize;

            let mut field_desc = format!("magic: sid={} (", sid);
            for sub in 0..np {
                let param_val = if sp > 0 { Some(r.read_u16(sp as u8)) } else { None };
                let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
                let raw_val;
                let adj_val;
                if vb <= 15 {
                    let v = r.read_u16(vb as u8);
                    raw_val = v;
                    adj_val = if sa != 0 { Some((v as i32).wrapping_sub(sa)) } else { None };
                } else {
                    let v = r.read_u32(vb as u8);
                    raw_val = v as u16;
                    adj_val = if sa != 0 { Some((v as i32).wrapping_sub(sa)) } else { None };
                }
                if sub > 0 { field_desc.push_str(" | "); }
                if let Some(pv) = param_val {
                    field_desc.push_str(&format!("p={},v={}", pv, raw_val));
                } else {
                    field_desc.push_str(&format!("v={}", raw_val));
                }
                if let Some(adj) = adj_val {
                    field_desc.push_str(&format!("->{}", adj));
                }
            }
            field_desc.push(')');
            eprintln!("  {:-4}..{:<4}  {}", mp_field_start, r.offset(), field_desc);
        }

        // ====== Chronicle ======
        let chr_start = orig(&r);
        // identified=1, so chronicle is skipped
        eprintln!("\n  {:-4}..{:<4}  chronicle (0 bits, identified=true → skip)", chr_start, chr_start);

        // ====== Chest-stackable trailer ======
        let tr_start = orig(&r);
        // not stackable page + not simple item → no trailer
        eprintln!("  {:-4}..{:<4}  chest-stackable trailer (0 bits, non-stackable page)", tr_start, tr_start);

        // ====== Align ======
        let al_start = orig(&r);
        r.align();
        eprintln!("  {:-4}..{:<4}  align to byte boundary", al_start, orig(&r));

        // ====== End of item / padding ======
        let end = orig(&r);
        let total_bits = raw.len() * 8;
        if end < total_bits {
            eprintln!("  {:-4}..{:<4}  [remaining padding / end of page data]", end, total_bits);
        }
        eprintln!("\n  Total page data: {} bits = {} bytes", total_bits, raw.len());
        eprintln!(" ======================================\n");
    }

    #[test]
    fn test_page0_trace_each() {
        use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(raw);
        let _jm = r.read_string(2);
        let cnt = r.read_u16(16);
        let total_bits = raw.len() * 8;
        eprintln!("Page[0]: {} items, {} bits ({:.1} avg/item)", cnt, total_bits, total_bits as f64 / cnt as f64);

        // For each item, track offset at start
        for i in 0..cnt {
            let start = r.offset();
            if start >= total_bits {
                eprintln!("[{:2}] EARLY END: bit {} >= {} total", i, start, total_bits);
                break;
            }
            match read_single_item(&mut r, 0, page.is_stackable) {
                Ok(item) => {
                    let code = item.item_type.trim().to_string();
                    let end = r.offset();
                    let bits = end - start;
                    let q = item.quality.unwrap_or(0);
                    let in_table = crate::protocol::d2i::legacy::game_items::ALL_ITEMS
                        .iter().any(|(c,_,_,_,_)| *c == code)
                        || crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP
                            .iter().any(|(c,_,_,_)| *c == code);
                    let marker = if !in_table { " ★ UNKNOWN" } else { "" };
                    eprintln!("[{:2}] {:>4}..{:<4} {:>4}b {:10} q={}{}", i, start, end, bits, code, q, marker);
                }
                Err(e) => {
                    eprintln!("[{:2}] ERROR at bit {}: {}", i, start, e);
                    break;
                }
            }
            // Stop if we've gone past the data
            if r.offset() >= total_bits { break; }
        }
        eprintln!("\nConsumed: {} bits / {} available", r.offset(), total_bits);
    }

    #[test]
    fn test_page0_gth_detail() {
        use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(raw);
        r.read_string(2); r.read_u16(16); // JM + count

        // Header 32b
        r.read_bit_array(4); r.read_bit(); r.read_bit_array(6);
        let soc = r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit_array(2);
        r.read_bit(); r.read_bit(); r.read_bit_array(3);
        r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit();
        let rw = r.read_bit(); r.read_bit_array(5);

        // Ver+loc
        let ver = r.read_u16(3); r.read_u8(3); r.read_u8(4);
        r.read_u8(4); r.read_u8(4); r.read_u8(3);

        // Huffman
        let code = crate::protocol::d2i::legacy::huffman::decode_huffman_string(&mut r).trim().to_string();
        let _sn = if false { r.read_u8(1) } else { r.read_u8(3) }; // non-simple

        eprintln!("item[0] code='{}' ver={} soc={} rw={}", code, ver, soc, rw);

        // Body
        let _iid = r.read_u32(32);
        let ilvl = r.read_u8(7);
        let _q = r.read_u8(4);
        let mp = r.read_bit(); if mp==1{ r.read_u8(3); }
        let cs = r.read_bit(); if cs==1{ r.read_u16(11); }

        // Unique quality
        let uid = r.read_u16(12);
        eprintln!("  uniq_id={} ilvl={} mp={} cs={}", uid, ilvl, mp, cs);

        // Conditional
        let (a,w,s) = lookup_item_category(&code);
        if a||s { let d=r.read_u16(11); eprintln!("  def={}", d); }
        if a||w||s { let md=r.read_u16(8); eprintln!("  maxdur={}", md); if md>0{let cd=r.read_u16(9); eprintln!("  curdur={}",cd);} }
        r.skip_bits(2);
        eprintln!("  v105_2b  at raw_offset={}", r.offset());

        // Magic Properties
        eprintln!("\n  Magic Properties:");
        loop {
            if r.offset()+9 > raw.len()*8 { break; }
            let sid = r.read_u16(9);
            if sid == 0x1FF { eprintln!("  {:>4}  0x1FF", r.offset()); break; }
            let su = sid as usize;
            if su >= MAGICAL_PROPS.len() { eprintln!("  {:>4}  UNKNOWN sid={} → skip 8b", r.offset()-9, sid); r.skip_bits(8); continue; }
            let p = &MAGICAL_PROPS[su];
            if p.save_bits==0 && p.save_param_bits==0 { r.read_bit(); continue; }
            let sb=p.save_bits as u8; let sp=p.save_param_bits as u8; let sa=p.save_add;
            let raw_start = r.offset()-9;
            for _ in 0..p.num_sub_props.max(1) {
                let pv = if sp>0{Some(r.read_u16(sp))}else{None};
                let vb = if sb>0{sb}else if sp>0{8}else{1};
                let val = if vb<=16{r.read_u16(vb)as u32}else{r.read_u32(vb)};
                let adj = (val as i32).wrapping_sub(sa);
                if let Some(par)=pv { eprintln!("  {:>4}..{:<4} sid={:<3} p={} v={} adj={}", raw_start, r.offset(), sid, par, val, adj); }
                else { eprintln!("  {:>4}..{:<4} sid={:<3} v={} adj={}", raw_start, r.offset(), sid, val, adj); }
            }
        }
        r.align();
        eprintln!("\n  Total: {} bits (from item start)", r.offset()-32);
    }

    #[test]
    fn test_page0_yme_detail() {
        use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(raw);
        r.read_string(2); r.read_u16(16); // JM + count

        // Skip item[0] gth
        r.seek(272);

        // item[1]
        let _h = r.offset(); r.read_bit_array(4);
        let identified = r.read_bit(); r.read_bit_array(6);
        r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit_array(2);
        r.read_bit(); r.read_bit(); r.read_bit_array(3);
        let simple = r.read_bit(); r.read_bit(); r.read_bit();
        let _pers = r.read_bit(); r.read_bit(); let _rw = r.read_bit(); r.read_bit_array(5);
        let _ver = r.read_u16(3); r.read_u8(3); r.read_u8(4); r.read_u8(4); r.read_u8(4); r.read_u8(3);
        let code = crate::protocol::d2i::legacy::huffman::decode_huffman_string(&mut r).trim().to_string();
        let _sn = if simple==1 { r.read_u8(1) } else { r.read_u8(3) };
        eprintln!("item[1]: code='{}' @ raw_offset={} id={} simple={}", code, _h, identified, simple);

        if simple == 0 {
            let _iid = r.read_u32(32); let _ilvl = r.read_u8(7); let _qval = r.read_u8(4);
            let _mp = r.read_bit(); if _mp==1{r.read_u8(3);}
            let _cs = r.read_bit(); if _cs==1{r.read_u16(11);}
            eprintln!("  quality={}", _qval);
            match _qval { 7 => {let uid=r.read_u16(12); eprintln!("  unique_id={}", uid);}
                6|8 => {r.read_u8(8);r.read_u8(8);for _ in 0..6{if r.read_bit()==1{r.read_u16(11);}}}
                5 => {let sid=r.read_u16(12); eprintln!("  set_id={}", sid);}
                _ => {}
            }
            let (a,w,s) = lookup_item_category(&code);
            if a||s{r.read_u16(11);} if a||w||s{let md=r.read_u16(8);if md>0{r.read_u16(9);}}
            r.skip_bits(2);
            eprintln!("  v105 @ {}", r.offset());

            eprintln!("\n  Magic Properties:");
            loop {
                if r.offset()+9 > raw.len()*8 { break; }
                let sid = r.read_u16(9);
                if sid == 0x1FF { eprintln!("  {:>4}  0x1FF", r.offset()); break; }
                let su = sid as usize;
                if su >= MAGICAL_PROPS.len() { eprintln!("  {:>4}  UNKNOWN sid={} → skip 8b", r.offset()-9, sid); r.skip_bits(8); continue; }
                let p = &MAGICAL_PROPS[su];
                if p.save_bits==0 && p.save_param_bits==0 { r.read_bit(); continue; }
                let sb=p.save_bits as u8; let sp=p.save_param_bits as u8; let sa=p.save_add;
                let rs = r.offset()-9;
                for _ in 0..p.num_sub_props.max(1) {
                    let pv = if sp>0{Some(r.read_u16(sp))}else{None};
                    let vb = if sb>0{sb}else if sp>0{8}else{1};
                    let val = if vb<=16{r.read_u16(vb)as u32}else{r.read_u32(vb)};
                    let adj = (val as i32).wrapping_sub(sa);
                    if let Some(par)=pv { eprintln!("  {:>4}..{:<4} sid={:<3} p={} v={} adj={}", rs, r.offset(), sid, par, val, adj); }
                    else { eprintln!("  {:>4}..{:<4} sid={:<3} v={} adj={}", rs, r.offset(), sid, val, adj); }
                }
            }
            r.align();
        }
        eprintln!("\n  Total item[1]: {} bits", r.offset()-272);
    }

    #[test]
    fn test_page0_each_body_size() {
        use crate::protocol::d2i::legacy::page::split_legacy_d2i_pages;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];

        // Parse each item manually, dump body_end and total
        let mut r = BitReader::new(raw);
        r.read_string(2); let declared = r.read_u16(16);
        eprintln!("Page[0]: declared={} items, {} bits available", declared, raw.len()*8);

        // Compute expected body offsets for common item types
        for i in 0..declared.min(80) {
            // Save position BEFORE each item
            if r.offset() >= raw.len()*8 { eprintln!("  [{}] EARLY END (no more data)", i); break; }
            let item_start = r.offset();
            let _item_start_byte = item_start / 8;

            // Read header bits
            r.read_bit_array(4); let id=r.read_bit(); r.read_bit_array(6);
            let _soc=r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit_array(2);
            r.read_bit(); r.read_bit(); r.read_bit_array(3);
            let simple=r.read_bit(); r.read_bit(); r.read_bit();
            let pers=r.read_bit(); r.read_bit(); let rw=r.read_bit(); r.read_bit_array(5);
            let _ver=r.read_u16(3); r.read_u8(3); r.read_u8(4); r.read_u8(4); r.read_u8(4); r.read_u8(3);
            let code = crate::protocol::d2i::legacy::huffman::decode_huffman_string(&mut r).trim().to_string();
            let _hint = if simple==1{r.read_u8(1)}else{r.read_u8(3)};

            let _before_body = r.offset();

            if simple == 0 {
                let _iid = r.read_u32(32); let _ilvl = r.read_u8(7);
                let qval = r.read_u8(4); let mp=r.read_bit();
                if mp==1{r.read_u8(3);}
                let cs=r.read_bit();
                if cs==1{let _aa=r.read_u16(11);}
                match qval {
                    5=>{let _sid=r.read_u16(12);}
                    6|8=>{let _r1=r.read_u8(8);let _r2=r.read_u8(8);
                          for _ai in 0..6{if r.read_bit()==1{let _id2=r.read_u16(11);}}
                    }
                    7=>{let _uid=r.read_u16(12);}
                    _=>{}
                }
                if rw==1{r.read_u16(12);r.read_u8(4);}
                if pers==1{for _ in 0..16{let c=r.read_u8(8);if c==0{break;}}}
                let (a,w,s)=lookup_item_category(&code);
                if a||s{r.read_u16(11);}
                if a||w||s{let md=r.read_u16(8);if md>0{r.read_u16(9);}}
                r.skip_bits(2);
                // Skip magic properties - scan for 0x1FF
                loop {
                    if r.offset()+9>raw.len()*8{break;}
                    let sid=r.read_u16(9);
                    if sid==0x1FF{break;}
                    let su=sid as usize;
                    if su>=MAGICAL_PROPS.len(){r.skip_bits(8);continue;}
                    let p=&MAGICAL_PROPS[su];
                    if p.save_bits==0&&p.save_param_bits==0{r.read_bit();continue;}
                    for _ in 0..p.num_sub_props.max(1){
                        if p.save_param_bits>0{r.skip_bits(p.save_param_bits as usize);}
                        let vb=if p.save_bits>0{p.save_bits as usize}else if p.save_param_bits>0{8}else{1};
                        if vb<=32{r.skip_bits(vb);}else{r.skip_bits(32);}
                    }
                }
                if id==0 && (qval==5||qval==7){r.skip_bits(52);}
            } else {
                let cs=r.read_bit();
                let _ = if cs==1{r.read_u8(8)}else{1};
            }
            r.align();
            let total_bits = r.offset() - item_start;
            let in_table = crate::protocol::d2i::legacy::game_items::ALL_ITEMS.iter().any(|(c,_,_,_,_)| *c == code.as_str())
                || crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP.iter().any(|(c,_,_,_)| *c == code.as_str());
            eprintln!("  [{:2}] {:>4}b  {:12} {}", i, total_bits, code,
                if in_table{"✓"}else{""});
        }
    }

    #[test]
    fn step2_count_10008000() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();

        let mut offset = 0usize;
        let mut page_idx = 0;
        loop {
            if offset + 64 > data.len() { break; }
            let magic = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
            if magic != 0xAA55AA55 { break; }

            let page_size = u32::from_le_bytes(data[offset+16..offset+20].try_into().unwrap()) as usize;
            let is_stackable = data[offset+20];
            let pd = &data[offset+64..offset+page_size];

            let mut r = BitReader::new(pd);
            let jm = r.read_string(2);
            if jm != "JM" {
                eprintln!("第{:2}页 (0x{:04X}) 无JM头",
                    page_idx, offset);
                offset += page_size; page_idx += 1; continue;
            }
            let count = r.read_u16(16);

            let mut total_unidentified = 0u32;
            let mut total_simple = 0u32;
            let mut total_identified_all = 0u32;
            let mut total_unique_jew = 0u32;

            for _ in 0..count {
                let start = r.offset();
                if start + 32 > pd.len() * 8 { break; }

                let b0 = r.read_u8(8); let _b1 = r.read_u8(8);
                let b2 = r.read_u8(8); let _b3 = r.read_u8(8);
                if ((b0 >> 4) & 1) == 1 { total_identified_all += 1; }
                if ((b0 >> 4) & 1) == 0 { total_unidentified += 1; }
                if ((b2 >> 5) & 1) == 1 { total_simple += 1; }
                // 跳过到下一个物品
                r.seek(start);
                match read_single_item(&mut r, 0, is_stackable == 1) {
                    Ok(ref item) => {
                        if item.item_type.trim() == "jew" && item.quality == Some(7) {
                            total_unique_jew += 1;
                        }
                    }
                    Err(_) => { break; }
                }
            }

            eprintln!("第{:2}页 (0x{:04X}) JM={:>4}  id_all={}  unid_all={}  simple={}  uniq_jew={}",
                page_idx, offset, count, total_identified_all, total_unidentified, total_simple, total_unique_jew);

            offset += page_size;
            page_idx += 1;
        }
    }

    /// 精确位级诊断:用真实 read_single_item 字段顺序,逐步打印 item[0] 字节
    /// 4..34 (前 30 字节) 每一段读到的值
    #[test]
    fn diagnostic_item0_field_by_field() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        use crate::protocol::d2i::legacy::huffman::decode_huffman_string;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(&raw[4..]); // 跳过 JM+count
        let start = r.offset(); // 应当是 0 (相对 item[0] 起点)
        let dump = |r: &BitReader, label: &str, raw: u32| {
            eprintln!("  raw_off={:>4}  rel_bit={:>4}  {} = {}",
                r.offset() / 8 * 8 + (r.offset() % 8), r.offset(), label, raw);
        };

        // ==== SECTION 1: 32b header ====
        eprintln!("=== SECTION 1: HEADER 32b ===");
        let h0 = r.read_bit_array(4); dump(&r, "b0_3", h0.iter().fold(0u32, |a,&b| a*2+b as u32));
        let identified = r.read_bit(); dump(&r, "identified", identified as u32);
        let h5 = r.read_bit_array(6); dump(&r, "b5_10", h5.iter().fold(0u32, |a,&b| a*2+b as u32));
        let socketed = r.read_bit(); dump(&r, "socketed", socketed as u32);
        let h12 = r.read_bit(); dump(&r, "b12", h12 as u32);
        let new_i = r.read_bit(); dump(&r, "new", new_i as u32);
        let h14 = r.read_bit_array(2); dump(&r, "b14_15", h14.iter().fold(0u32, |a,&b| a*2+b as u32));
        let is_ear = r.read_bit(); dump(&r, "is_ear", is_ear as u32);
        let starter = r.read_bit(); dump(&r, "starter", starter as u32);
        let h18 = r.read_bit_array(3); dump(&r, "b18_20", h18.iter().fold(0u32, |a,&b| a*2+b as u32));
        let simple = r.read_bit(); dump(&r, "simple", simple as u32);
        let eth = r.read_bit(); dump(&r, "ethereal", eth as u32);
        let h23 = r.read_bit(); dump(&r, "b23", h23 as u32);
        let pers = r.read_bit(); dump(&r, "personalized", pers as u32);
        let h25 = r.read_bit(); dump(&r, "b25", h25 as u32);
        let rw = r.read_bit(); dump(&r, "given_runeword", rw as u32);
        let h27 = r.read_bit_array(5); dump(&r, "b27_31", h27.iter().fold(0u32, |a,&b| a*2+b as u32));
        eprintln!("  → identified={} socketed={} simple={} rw={} pers={}",
            identified, socketed, simple, rw, pers);

        // ==== SECTION 2: ver+loc ====
        eprintln!("=== SECTION 2: VER+LOC 21b ===");
        let ver = r.read_u16(3) as u32; dump(&r, "version", ver);
        let loc = r.read_u8(3) as u32; dump(&r, "location", loc);
        let eq = r.read_u8(4) as u32; dump(&r, "equipped", eq);
        let px = r.read_u8(4) as u32; dump(&r, "pos_x", px);
        let py = r.read_u8(4) as u32; dump(&r, "pos_y", py);
        let alt = r.read_u8(3) as u32; dump(&r, "alt_pos", alt);

        // ==== SECTION 2.5: huffman 4 char ====
        eprintln!("=== HUFFMAN 4-char ===");
        let h_start = r.offset();
        let code = decode_huffman_string(&mut r);
        eprintln!("  raw_bit={}..{}  code='{}' (trimmed)", h_start, r.offset(), code.trim());

        // ==== hint ====
        eprintln!("=== HINT 3b (non-simple) ===");
        let hint = r.read_u8(3) as u32; dump(&r, "sockets_hint", hint);

        // ==== SECTION 3: body ====
        eprintln!("=== SECTION 3: BODY (all fields, no skip) ===");
        let _b0 = r.offset();
        let iid = r.read_u32(32); dump(&r, "item_id", iid);
        let lvl = r.read_u8(7) as u32; dump(&r, "level", lvl);
        let qval = r.read_u8(4) as u32; dump(&r, "quality", qval);
        let qnames = ["Low","Normal","Superior","Magic","Set","Rare","Unique","Crafted"];
        eprintln!("    quality={} → {}", qval, qnames.get(qval as usize).unwrap_or(&"?"));
        let mp = r.read_bit(); dump(&r, "multi_pic", mp as u32);
        if mp == 1 { let pid = r.read_u8(3) as u32; dump(&r, "  picture_id", pid); }
        let cs = r.read_bit(); dump(&r, "class_specific", cs as u32);
        if cs == 1 { let aa = r.read_u16(11) as u32; dump(&r, "  auto_affix_id", aa); }

        match qval {
            1 => { let lid = r.read_u8(3) as u32; dump(&r, "  Low_id", lid); }
            3 => { let fi = r.read_u8(3) as u32; dump(&r, "  Superior_flag", fi); }
            4 => {
                let pre = r.read_u16(11) as u32; dump(&r, "  Magic_prefix_id", pre);
                let suf = r.read_u16(11) as u32; dump(&r, "  Magic_suffix_id", suf);
            }
            5 => { let sid = r.read_u16(12) as u32; dump(&r, "  Set_id", sid); }
            6 => { let uid = r.read_u16(12) as u32; dump(&r, "  Unique_id", uid); }
            7 | 8 => {
                let r1 = r.read_u8(8) as u32; dump(&r, "  Rare_name1", r1);
                let r2 = r.read_u8(8) as u32; dump(&r, "  Rare_name2", r2);
                for ai in 0..6 {
                    let f = r.read_bit();
                    if f == 1 { let m = r.read_u16(11) as u32; dump(&r, &format!("  affix[{}]_id", ai), m); }
                    else { dump(&r, &format!("  affix[{}]_flag", ai), 0); }
                }
            }
            _ => eprintln!("  (qval={} no quality-specific fields)", qval),
        }
        if rw == 1 { let rid = r.read_u16(12) as u32; dump(&r, "  rw_id", rid); let rp = r.read_u8(4) as u32; dump(&r, "  rw_pad", rp); }
        if pers == 1 {
            eprintln!("  personalized 16 chars × 8b:");
            for ci in 0..16 { let c = r.read_u8(8) as u32; if c == 0 { break; } eprint!("    c[{}]={} ", ci, c); }
            eprintln!();
        }
        // Note: lookup_item_category for unknown 'gth' returns (false,false,false)
        eprintln!("  (gth not in tables → no def/dur)");
        r.skip_bits(2);
        eprintln!("  v105_2b at rel_bit={}", r.offset());

        eprintln!("=== item[0] consumed rel_bits = {} (= {} bytes) ===", r.offset(), r.offset() as f64 / 8.0);
        let _ = start;
    }

    /// 跟踪 item[0] 的 magic properties,定位 0x1FF 终止符 + 真实总长
    #[test]
    #[ignore = "诊断测试，依赖特定 stash 文件中的 item 数据长度，非功能测试"]
    fn diagnostic_item0_magic_props() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        use crate::protocol::d2i::legacy::huffman::decode_huffman_string;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(&raw[4..]); // 跳过 JM+count

        // header 32b
        r.read_bit_array(4); r.read_bit(); r.read_bit_array(6);
        r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit_array(2);
        r.read_bit(); r.read_bit(); r.read_bit_array(3);
        r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit();
        r.read_bit(); r.read_bit_array(5);
        // ver+loc 21b
        r.read_u16(3); r.read_u8(3); r.read_u8(4);
        r.read_u8(4); r.read_u8(4); r.read_u8(3);
        // huffman 4 char
        decode_huffman_string(&mut r);
        // hint 3b
        r.read_u8(3);
        // body
        r.read_u32(32); r.read_u8(7); r.read_u8(4);
        r.read_bit(); // mp
        r.read_bit(); // cs
        // quality 7 = Unique → 12b uniq_id
        r.read_u16(12);
        eprintln!("after uniq_id at rel_bit={} (byte {})", r.offset(), r.offset()/8);

        // 防御/耐久 (gth not in table → 0)
        r.skip_bits(2);
        eprintln!("v105_2b at rel_bit={} (byte {})", r.offset(), r.offset()/8);

        // 读 magic properties,精确记录每个 stat 起点
        let mut n = 0;
        loop {
            if r.offset() + 9 > r.len_bits() { eprintln!("  ran out of data"); break; }
            let sid = r.read_u16(9);
            let sid_start = r.offset() - 9;
            if sid == 0x1FF {
                eprintln!("  rel_bit={:>4} (byte {:>4})  0x1FF TERMINATOR ← end of magic", r.offset(), r.offset()/8);
                break;
            }
            let su = sid as usize;
            eprint!("  rel_bit={:>4} (byte {:>4})  sid={:>3} ", sid_start, sid_start/8, sid);
            if su >= MAGICAL_PROPS.len() {
                r.skip_bits(8);
                eprintln!("[UNKNOWN, skip 8b]");
            } else {
                let p = &MAGICAL_PROPS[su];
                let sb = p.save_bits;
                let sp = p.save_param_bits;
                let sa = p.save_add;
                let np = p.num_sub_props.max(1) as usize;
                if sb == 0 && sp == 0 {
                    r.read_bit();
                    eprintln!("[character-only, skip 1b]");
                } else {
                    for _ in 0..np {
                        if sp > 0 { r.skip_bits(sp as usize); }
                        let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
                        if vb <= 15 { r.read_u16(vb as u8); } else { r.read_u32(vb as u8); }
                    }
                    eprintln!("[sb={} sa={} sp={} np={}]", sb, sa, sp, np);
                }
            }
            n += 1;
            if n > 30 { eprintln!("  too many stats, breaking"); break; }
        }
        let end = r.offset();
        eprintln!("\nmagic properties ends at rel_bit={}", end);
        eprintln!("0x1FF occupied bits 263..272 (byte 32 bit 7 + byte 33)");
        eprintln!("item[0] body end = 272 (= rel_bit 240 from item_start)");
        eprintln!("after align(), reader offset = {}", (end + 7) & !7);
        eprintln!("next item would start at rel_bit = {}", (end + 7) & !7);

        // 显示下 16 字节 (item[1] 起点附近)
        eprintln!("\nNext 16 bytes of raw[4..] starting from byte {}:", end / 8);
        let start = end / 8;
        for i in 0..16 {
            let b = raw[4 + start + i];
            print!("{:02x} ", b);
        }
        println!();
    }

    /// 调查 0x1FF 后 8b 扩展字段:对 item[0]..[10] 各读 0x1FF 后 32b,
    /// 同时统计该物品是否有 coi_inf_t*_count/gate (sid 368-374) magic stat
    #[test]
    fn investigate_post_0x1ff_8b_field() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        use crate::protocol::d2i::legacy::huffman::decode_huffman_string;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];
        let mut r = BitReader::new(&raw[4..]);
        r.read_string(2); r.read_u16(16); // JM + count

        // 跑前 12 个物品,记录每个的:
        //   - code
        //   - 是否含 coi_inf stat (sid 368-374)
        //   - 0x1FF 后 8b 的值
        //   - 0x1FF 后 32b 的 raw bytes
        for idx in 0..12 {
            let item_start = r.offset();
            eprintln!("\n========= item[{}] start at rel_bit={} (byte {}) =========", idx, item_start, item_start/8);
            // header 32b
            r.read_bit_array(4); r.read_bit(); r.read_bit_array(6);
            r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit_array(2);
            r.read_bit(); r.read_bit(); r.read_bit_array(3);
            r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit(); r.read_bit();
            r.read_bit(); r.read_bit_array(5);
            // ver+loc 21b
            r.read_u16(3); r.read_u8(3); r.read_u8(4);
            r.read_u8(4); r.read_u8(4); r.read_u8(3);
            // huffman 4 char
            let code = decode_huffman_string(&mut r).trim().to_string();
            // hint 3b
            r.read_u8(3);
            // body
            r.read_u32(32); r.read_u8(7); r.read_u8(4);
            let qval = (r.offset(), ); r.read_bit(); r.read_bit();
            let _ = qval;
            // quality-specific (假设走 real skip,实际我们不知道)
            // 简化: 直接尝试所有 4b 值的路径
            // 但这里为了快速调查,我们只走一个保守的扫描 — 直接读 magic properties
            // 用一种最 robust 方式:尝试不同 quality,直到 0x1FF 出现
            // 简单方案:硬编码 item[0] 已知是 Unique (q=7),其他 跳过 skip_non_simple
            // 实际上为了"0x1FF 后 8b"调查,直接调用 real read_single_item
            // 但我们需要中途读 sid 列表。改用 real read_single_item + 0x1FF 后 peek 32b
            let _item_start_byte = item_start / 8;
            eprintln!("  code='{}' (skipped body)", code);
            // 跳过剩余 body
            // 算了,改用调用 read_single_item 解析整个,然后 peek 0x1FF 后字节
            // 但这要求我们从 item_start 重新来
            r.seek(item_start);
            match read_single_item(&mut r, 0, false) {
                Ok(item) => {
                    let end = r.offset();
                    eprintln!("  parsed: type='{}' q={:?} bits={}", item.item_type, item.quality, end - item_start);
                    // peek 32b 0x1FF 后:实际 0x1FF 已被 read_single_item 消费
                    // 改用从 item_data 重新扫描
                    // 找最近的 0x1FF (item_start 后)
                    // 简化:直接显示 raw[4..] 在 end 位置起的 32 字节
                    let start_byte = end / 8;
                    if start_byte + 32 <= raw.len() - 4 {
                        eprint!("  bytes after item[{}] (raw[4..] byte {}..{}): ", idx, start_byte, start_byte + 32);
                        for i in 0..32 {
                            let b = raw[4 + start_byte + i];
                            print!("{:02x} ", b);
                        }
                        println!();
                    }
                }
                Err(e) => eprintln!("  ERR: {}", e),
            }
        }
    }

    /// Page[3] amulet 项链(已知 sid 368-373 注灵 stat)的 0x1FF 后 8b 调查
    /// 目标:看 8b 值是否与 coi_inf stat 数量/gate 值关联
    #[test]
    fn investigate_page3_amulet_8b() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[3];
        let raw = &page.data[64..];

        eprintln!("=== Page[3] raw bytes (前 {} 字节) ===", raw.len().min(80));
        for i in 0..raw.len().min(80) {
            print!("{:02x} ", raw[i]);
            if (i + 1) % 16 == 0 { println!(); }
        }
        println!();

        // 跑 read_single_item 解析整个
        let mut r = BitReader::new(raw);
        let _jm = r.read_string(2);
        let cnt = r.read_u16(16);
        eprintln!("JM='{}' count={}", _jm, cnt);
        let item_start = r.offset();
        eprintln!("item[0] start at rel_bit={} (byte {})", item_start, item_start/8);
        let res = read_single_item(&mut r, 0, page.is_stackable);
        let end_bit = r.offset();
        eprintln!("read_single_item done: end_bit={} (byte {}), bits={}",
            end_bit, end_bit/8, end_bit - item_start);
        if let Ok(ref item) = res {
            eprintln!("  parsed: type='{}' q={:?}", item.item_type, item.quality);
        }

        // 收集 sid 列表:用真实 read_single_item 之后,重新扫描
        // 简单做法:从 item_start 开始重新按 q=Rare 路径读
        let mut r2 = BitReader::new(raw);
        let _ = r2.read_string(2); r2.read_u16(16);
        // header 32b
        r2.read_bit_array(4); r2.read_bit(); r2.read_bit_array(6);
        r2.read_bit(); r2.read_bit(); r2.read_bit(); r2.read_bit_array(2);
        r2.read_bit(); r2.read_bit(); r2.read_bit_array(3);
        r2.read_bit(); r2.read_bit(); r2.read_bit(); r2.read_bit(); r2.read_bit();
        r2.read_bit(); r2.read_bit_array(5);
        // ver+loc 21b
        r2.read_u16(3); r2.read_u8(3); r2.read_u8(4);
        r2.read_u8(4); r2.read_u8(4); r2.read_u8(3);
        // huffman
        let code = crate::protocol::d2i::legacy::huffman::decode_huffman_string(&mut r2).trim().to_string();
        eprintln!("huffman code = '{}'", code);
        r2.read_u8(3);
        // body
        let _iid = r2.read_u32(32);
        let _ilvl = r2.read_u8(7);
        let _q = r2.read_u8(4);
        r2.read_bit(); r2.read_bit();
        // Rare 路径
        let r1 = r2.read_u8(8) as u32;
        let r2n = r2.read_u8(8) as u32;
        eprintln!("rare_name1={} rare_name2={}", r1, r2n);
        let mut affix_count = 0;
        for ai in 0..6 {
            let f = r2.read_bit();
            if f == 1 {
                let m = r2.read_u16(11) as u32;
                let aff_type = if m < 728 { "prefix" } else { "suffix" };
                eprintln!("  affix[{}] id={} ({})", ai, m, aff_type);
                affix_count += 1;
            }
        }
        eprintln!("rare affix count = {}", affix_count);
        r2.skip_bits(2); // v105
        eprintln!("v105_2b at rel_bit={}", r2.offset());

        // 读 magic properties
        eprintln!("\n--- MAGIC PROPERTIES ---");
        let mut sids: Vec<u16> = Vec::new();
        loop {
            if r2.offset() + 9 > r2.len_bits() { break; }
            let sid = r2.read_u16(9);
            if sid == 0x1FF {
                eprintln!("rel_bit={} (byte {})  0x1FF TERMINATOR", r2.offset(), r2.offset()/8);
                break;
            }
            sids.push(sid);
            let su = sid as usize;
            if su >= MAGICAL_PROPS.len() {
                eprintln!("rel_bit={}  sid={} [UNKNOWN, skip 8b]", r2.offset()-9, sid);
                r2.skip_bits(8);
                continue;
            }
            let p = &MAGICAL_PROPS[su];
            if p.save_bits == 0 && p.save_param_bits == 0 {
                eprintln!("rel_bit={}  sid={} [character-only, 1b]", r2.offset()-9, sid);
                r2.read_bit();
                continue;
            }
            let sb = p.save_bits;
            let sp = p.save_param_bits;
            let sa = p.save_add;
            let np = p.num_sub_props.max(1) as usize;
            for _ in 0..np {
                if sp > 0 { r2.skip_bits(sp as usize); }
                let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
                if vb <= 15 { r2.read_u16(vb as u8); } else { r2.read_u32(vb as u8); }
            }
            let _ = sa;
            eprintln!("rel_bit={} (byte {})  sid={} [sb={} sa={} sp={} np={}]",
                r2.offset(), r2.offset()/8, sid, sb, sa, sp, np);
        }
        let magic_end = r2.offset();
        eprintln!("\n0x1FF ends at rel_bit={} (byte {})", magic_end, magic_end/8);

        // 0x1FF 后 8b
        let post8b_start = magic_end;
        let post8b_byte = post8b_start / 8;
        let post8b_bit_offset = post8b_start % 8;
        eprintln!("\n0x1FF 后 8b 起点: rel_bit={} (byte {} bit {})",
            post8b_start, post8b_byte, post8b_bit_offset);
        eprintln!("0x1FF 后 32 字节:");
        for i in 0..32 {
            if post8b_byte + i < raw.len() {
                print!("{:02x} ", raw[post8b_byte + i]);
            }
        }
        println!();

        // 统计 sid 368-374 (coi_inf) 数量
        let coi_inf_sids: Vec<u16> = sids.iter().copied()
            .filter(|s| (368..=374).contains(s)).collect();
        eprintln!("\ncoi_inf stat (368-374) 数量: {}", coi_inf_sids.len());
        eprintln!("coi_inf sids: {:?}", coi_inf_sids);

        // 解析这 8b:看 8 个 bit 每一位的值
        if post8b_bit_offset == 0 && post8b_byte < raw.len() {
            let b = raw[post8b_byte];
            eprintln!("\n0x1FF 后 8b 整字节 = 0x{:02x} = 0b{:08b}", b, b);
        } else if post8b_bit_offset == 0 {
            eprintln!("\n(0x1FF 后 8b 超出 raw 范围: byte={}, raw.len={})", post8b_byte, raw.len());
        } else {
            eprintln!("\n(0x1FF 后 8b 跨字节,bit_offset={})", post8b_bit_offset);
        }
    }

    /// 完整 dump gth 装备(Page[0] item[0])的所有数据
    #[test]
    fn dump_gth_full() {
        use crate::protocol::d2i::legacy::bit_reader::BitReader;
        use crate::protocol::d2i::legacy::huffman::decode_huffman_string;
        let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
        let data = std::fs::read(path).unwrap();
        let pages = split_legacy_d2i_pages(&data).unwrap();
        let page = &pages.pages[0];
        let raw = &page.data[64..];

        let mut r = BitReader::new(raw);
        let _jm = r.read_string(2); r.read_u16(16);
        let item_start = r.offset();
        eprintln!("=== gth (Page[0] item[0]) full dump ===");
        eprintln!("item_start: rel_bit={} (byte {})", item_start, item_start/8);

        // SECTION 1: 32b header
        eprintln!("\n--- SECTION 1: 32b header ---");
        let b0_3 = r.read_bit_array(4);
        let identified = r.read_bit();
        let b5_10 = r.read_bit_array(6);
        let socketed = r.read_bit();
        let b12 = r.read_bit();
        let new = r.read_bit();
        let b14_15 = r.read_bit_array(2);
        let is_ear = r.read_bit();
        let starter = r.read_bit();
        let b18_20 = r.read_bit_array(3);
        let simple = r.read_bit();
        let ethereal = r.read_bit();
        let b23 = r.read_bit();
        let personalized = r.read_bit();
        let b25 = r.read_bit();
        let given_runeword = r.read_bit();
        let b27_31 = r.read_bit_array(5);
        eprintln!("identified={} socketed={} simple={} ethereal={} b23={} personalized={} given_runeword={}",
            identified, socketed, simple, ethereal, b23, personalized, given_runeword);
        eprintln!("b0_3={:?} b5_10={:?} b12={} new={} b14_15={:?} is_ear={} starter={} b18_20={:?} b25={} b27_31={:?}",
            b0_3, b5_10, b12, new, b14_15, is_ear, starter, b18_20, b25, b27_31);

        // SECTION 2: 21b ver+loc
        eprintln!("\n--- SECTION 2: 21b ver+loc ---");
        let ver = r.read_u16(3);
        let loc = r.read_u8(3);
        let equipped = r.read_u8(4);
        let pos_x = r.read_u8(4);
        let pos_y = r.read_u8(4);
        let alt_pos = r.read_u8(3);
        eprintln!("version={} location={} equipped={} pos_x={} pos_y={} alt_pos={}",
            ver, loc, equipped, pos_x, pos_y, alt_pos);

        // huffman
        let code = decode_huffman_string(&mut r);
        eprintln!("\nhuffman code (4 chars) = '{:?}' trimmed='{}'", code, code.trim());

        // hint 3b (non-simple)
        let hint = r.read_u8(3);
        eprintln!("hint (3b non-simple) = {}", hint);

        // SECTION 3: body
        eprintln!("\n--- SECTION 3: body ---");
        let item_id = r.read_u32(32);
        let ilvl = r.read_u8(7);
        let quality = r.read_u8(4);
        eprintln!("item_id={} ({:#010x})", item_id, item_id);
        eprintln!("ilvl={} quality={}", ilvl, quality);
        let qnames = ["(0)","Low","Normal","Superior","Magic","Set","Rare","Unique","Crafted"];
        eprintln!("  → quality = {} ({})", quality, qnames.get(quality as usize).copied().unwrap_or("?"));

        let mp = r.read_bit();
        eprintln!("multi_pic={}", mp);
        if mp == 1 { let pid = r.read_u8(3); eprintln!("  picture_id={}", pid); }
        let cs = r.read_bit();
        eprintln!("class_specific={}", cs);
        if cs == 1 { let aa = r.read_u16(11); eprintln!("  auto_affix_id={}", aa); }

        // Quality-specific: q=7 Unique
        if quality == 7 {
            let uniq_id = r.read_u16(12);
            eprintln!("\n  Unique uniq_id = {} ({:#x})", uniq_id, uniq_id);
        } else if quality == 5 {
            let set_id = r.read_u16(12);
            eprintln!("\n  Set set_id = {} ({:#x})", set_id, set_id);
        } else if quality == 6 || quality == 8 {
            let rn1 = r.read_u8(8);
            let rn2 = r.read_u8(8);
            eprintln!("\n  Rare/Crafted rare_name1={} rare_name2={}", rn1, rn2);
        } else if quality == 4 {
            let pre = r.read_u16(11);
            let suf = r.read_u16(11);
            eprintln!("\n  Magic pre={} suf={}", pre, suf);
        } else if quality == 3 {
            let fi = r.read_u8(3);
            eprintln!("\n  Superior flag = {}", fi);
        } else if quality == 1 {
            let lid = r.read_u8(3);
            eprintln!("\n  Low id = {}", lid);
        }

        // given_runeword / personalized
        if given_runeword == 1 {
            let rid = r.read_u16(12);
            let rp = r.read_u8(4);
            eprintln!("  rw_id={} rw_pad={}", rid, rp);
        }
        if personalized == 1 {
            eprintln!("  personalized:");
            for ci in 0..16 {
                let c = r.read_u8(8);
                if c == 0 { break; }
                eprint!(" c[{}]={} ", ci, c);
            }
            eprintln!();
        }

        // defense / durability
        eprintln!("\n--- 防御/耐久 (gth = armor) ---");
        if quality == 7 { // Unique
            // gth 是 armor,lookup 应当返回 (true, false, false)
            eprintln!("  (gth 是 armor,应读 11b def + 8b md)");
        }
        let def = r.read_u16(11);
        let md = r.read_u8(8);
        eprintln!("  def = {}  max_durability = {}", def, md);
        if md > 0 {
            let cd = r.read_u16(9);
            eprintln!("  cur_durability = {}", cd);
        }

        // v105 2b
        r.skip_bits(2);
        eprintln!("\n--- v105 2b skipped ---");
        eprintln!("magic properties 起点: rel_bit={} (byte {})", r.offset(), r.offset()/8);

        // magic properties
        eprintln!("\n--- MAGIC PROPERTIES ---");
        loop {
            if r.offset() + 9 > r.len_bits() { eprintln!("  [end of data]"); break; }
            let sid = r.read_u16(9);
            if sid == 0x1FF {
                eprintln!("  rel_bit={}  0x1FF TERMINATOR ← end", r.offset());
                break;
            }
            let su = sid as usize;
            let name = ["str","eng","dex","vit","pts","skl","hp","maxhp","mana","maxmana","sta","maxsta","lvl","exp","gld","gldb",
                "armor%","maxdmg%","mindmg%","tohit","toblock","mindmg","maxdmg","2mindmg","2maxdmg","dmg%","manarec",
                "manarec%","stamrec%","lastexp","nextexp","armorclass","acvsmiss","acvshth","normdmgr","magicdmgr",
                "damageresist","magicresist","maxmagres","fireresist","maxfireres","lightresist","maxlightres",
                "coldresist","maxcoldres","poisonresist","maxpoisonres","dmgaura","firemindam","firemaxdam",
                "lightmindam","lightmaxdam","magicmindam","magicmaxdam","coldmindam","coldmaxdam","coldlength",
                "poisonmindam","poisonmaxdam","poisonlength","lifedrainmindam","lifedrainmaxdam","manadrainmindam",
                "manadrainmaxdam","stamdrainmindam","stamdrainmaxdam","staminarecovery","velocitypercent",
                "attackrate","other_animrate","quantity","value","durability","maxdurability","hpregen",
                "item_maxdurability_percent","item_maxhp_percent","item_maxmana_percent","item_attackertakesdamage",
                "item_goldbonus","item_magicbonus","item_knockback","item_timeduration","item_addclassskills",
                "item_addclassskills2","item_addexperience","item_healafterkill","item_healafterkill_percent",
                "item_doubleherbduration","item_lightradius","item_lightcolor","item_req_percent",
                "item_levelreq","item_fasterattackrate","item_levelreq_percent","last_block_frame",
                "item_fastermovevelocity","item_nonclassskill","state","item_fastergethitrate"];
            let sname = if su < name.len() { name[su] } else { "?" };
            if su >= MAGICAL_PROPS.len() {
                eprintln!("  sid={} [UNKNOWN mod stat, skip 8b]", sid);
                r.skip_bits(8);
                continue;
            }
            let p = &MAGICAL_PROPS[su];
            if p.save_bits == 0 && p.save_param_bits == 0 {
                eprintln!("  sid={} ({}) [character-only, 1b]", sid, sname);
                r.read_bit();
                continue;
            }
            let sb = p.save_bits;
            let sp = p.save_param_bits;
            let sa = p.save_add;
            let np = p.num_sub_props.max(1) as usize;
            let mut desc = format!("  sid={} ({}) [sb={} sa={} sp={} np={}] values:",
                sid, sname, sb, sa, sp, np);
            for _ in 0..np {
                if sp > 0 { r.skip_bits(sp as usize); }
                let vb = if sb > 0 { sb } else if sp > 0 { 8 } else { 1 };
                let raw = if vb <= 15 { r.read_u16(vb as u8) as u32 }
                          else { r.read_u32(vb as u8) };
                let adj = if sa != 0 { (raw as i32).wrapping_sub(sa) } else { raw as i32 };
                desc.push_str(&format!(" raw={} adj={}", raw, adj));
            }
            eprintln!("{}", desc);
        }
        let magic_end = r.offset();
        eprintln!("\n0x1FF 终点: rel_bit={} (byte {})", magic_end, magic_end/8);
        let gth_len = magic_end - item_start;
        eprintln!("gth 实际长度: {}b (={} 字节)", gth_len, gth_len as f64 / 8.0);

        // 0x1FF 后 8b
        eprintln!("\n--- 0x1FF 后 8b 验证 ---");
        let post8b_byte = magic_end / 8;
        if post8b_byte < raw.len() {
            let b = raw[post8b_byte];
            eprintln!("0x1FF 后 1 byte = 0x{:02x} = 0b{:08b}", b, b);
            eprintln!("0x1FF 后 8b (此 byte 8 bit):");
            for bi in 0..8 {
                eprintln!("  bit[{}] = {}", bi, (b >> bi) & 1);
            }
        }

        // 显示下 32 字节 (item[1] 起点附近)
        eprintln!("\n--- 0x1FF 后 32 字节 (item[1] 起点附近) ---");
        for i in 0..32 {
            if post8b_byte + i < raw.len() {
                print!("{:02x} ", raw[post8b_byte + i]);
            }
        }
        println!();
    }
}
