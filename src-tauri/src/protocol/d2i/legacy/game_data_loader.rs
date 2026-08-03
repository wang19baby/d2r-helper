/// Runtime game data loader from D2R excel TXT files.
///
/// Reads misc.txt / armor.txt / weapons.txt at runtime to provide:
/// - Accurate inventory sizes (invwidth/invheight)
/// - Stackable flag per item
/// - Item type classification (rune, gem, helm, axe, etc.)
/// - Mod-added item detection (diff against hardcoded ALL_ITEMS)
///
/// Falls back gracefully when TXT files aren't available.
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use super::game_items::ALL_ITEMS;

/// Runtime-loaded item definition
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub code: String,
    pub name: String,
    pub inv_width: u8,
    pub inv_height: u8,
    pub item_type: String,    // from `type` column ("rune", "gem", "helm", "axe"…)
    pub stackable: bool,
    pub is_mod_item: bool,    // true if code NOT in hardcoded ALL_ITEMS
}

/// Cached game data loaded from TXT files
#[derive(Debug)]
pub struct ItemData {
    pub items: HashMap<String, ItemDef>,
    /// code → (width, height) quick lookup
    inv_sizes: HashMap<String, (u8, u8)>,
    /// set of stackable codes
    stackable_set: HashSet<String>,
    /// set of codes NOT in vanilla ALL_ITEMS
    mod_codes: HashSet<String>,
}

static ITEM_DATA: OnceLock<ItemData> = OnceLock::new();

/// Grimoires are modded offhand books used in the shield/offhand slot.
/// They often carry defense like armor, but their equipment slot semantics
/// are closer to necro heads / offhands than body armor.
pub fn is_grimoire_offhand(code: &str) -> bool {
    matches!(
        code.trim().to_lowercase().as_str(),
        "wa1" | "wa2" | "wa3" | "wa4" | "wa5"
            | "wa6" | "wa7" | "wa8" | "wa9" | "waa" | "wab" | "wac" | "wad" | "wae" | "waf"
    )
}

/// Initialize the global item data cache from game TXT directory.
/// Safe to call multiple times — only the first call loads.
pub fn initialize(game_data_path: &str) {
    if ITEM_DATA.get().is_some() {
        return; // already initialized
    }
    let dir = Path::new(game_data_path);
    if !dir.exists() || !dir.is_dir() {
        log::warn!("[game_data_loader] path not found: {}", game_data_path);
        return;
    }

    // Build vanilla code set for mod detection
    let vanilla_codes: HashSet<&str> = ALL_ITEMS.iter().map(|(c, _, _, _, _)| *c).collect();

    // Load TXT files
    let mut items: HashMap<String, ItemDef> = HashMap::new();
    let mut inv_sizes: HashMap<String, (u8, u8)> = HashMap::new();
    let mut stackable_set: HashSet<String> = HashSet::new();
    let mut mod_codes: HashSet<String> = HashSet::new();

    for filename in &["misc.txt", "armor.txt", "weapons.txt"] {
        // Try base/ subdirectory first, then root
        let p = dir.join("base").join(filename);
        let p = if p.exists() { p } else { dir.join(filename) };
        if !p.exists() {
            log::debug!("[game_data_loader] {} not found in {}", filename, game_data_path);
            continue;
        }

        log::info!("[game_data_loader] loading {}", p.display());
        if let Some(rows) = read_txt_file(&p) {
            for row in rows {
                let code = match row.get("code") {
                    Some(c) => c.trim().to_lowercase(),
                    None => continue,
                };
                if code.is_empty() {
                    continue;
                }

                let name = row.get("name").map(|s| s.to_string()).unwrap_or_default();
                // If the TXT row omits invwidth/invheight, fall back to the
                // hardcoded ITEM_INVENTORY_SIZES table rather than defaulting
                // to (1, 1). Many vanilla rows (e.g. kit) have these columns
                // missing, and (1, 1) would shadow the correct hardcoded value
                // (kit is 2x3) and break downstream placement logic.
                let hardcoded_size = super::item_sizes::ITEM_INVENTORY_SIZES
                    .iter()
                    .find(|(c, _, _)| *c == code)
                    .map(|(_, w, h)| (*w, *h));
                let inv_width = row.get("invwidth")
                    .and_then(|v| v.parse::<u8>().ok())
                    .or_else(|| hardcoded_size.map(|(w, _)| w))
                    .unwrap_or(1);
                let inv_height = row.get("invheight")
                    .and_then(|v| v.parse::<u8>().ok())
                    .or_else(|| hardcoded_size.map(|(_, h)| h))
                    .unwrap_or(1);
                let item_type = row.get("type").or_else(|| row.get("*type")).map(|s| s.to_string()).unwrap_or_default();
                let stackable = row.get("stackable").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) == 1;

                let is_mod_item = !vanilla_codes.contains(code.as_str());

                items.insert(code.clone(), ItemDef {
                    code: code.clone(),
                    name,
                    inv_width,
                    inv_height,
                    item_type,
                    stackable,
                    is_mod_item,
                });
                inv_sizes.insert(code.clone(), (inv_width, inv_height));
                if stackable {
                    stackable_set.insert(code.clone());
                }
                if is_mod_item {
                    mod_codes.insert(code);
                }
            }
        }
    }

    let data = ItemData { items, inv_sizes, stackable_set, mod_codes };
    log::info!(
        "[game_data_loader] loaded {} items ({} vanilla, {} mod-added) from TXT",
        data.items.len(),
        data.items.len() - data.mod_codes.len(),
        data.mod_codes.len(),
    );

    // Silently ignore if already set (race condition safe)
    let _ = ITEM_DATA.set(data);
}

/// Check if the item data has been loaded
pub fn is_loaded() -> bool {
    ITEM_DATA.get().is_some()
}

/// Get inventory size for a code: runtime → hardcoded → category default
pub fn get_inventory_size(code: &str) -> (u8, u8) {
    let key = code.trim().to_lowercase();
    if is_grimoire_offhand(&key) {
        log::debug!("[get_inventory_size] code={} source=grimoire_offhand size=(2,2)", key);
        return (2, 2);
    }
    // Runtime (TXT) first
    if let Some(data) = ITEM_DATA.get()
        && let Some(&size) = data.inv_sizes.get(&key) {
            log::debug!("[get_inventory_size] code={} source=runtime_txt size=({},{})", key, size.0, size.1);
            return size;
        }
    // Fallback to hardcoded table
    for (c, w, h) in super::item_sizes::ITEM_INVENTORY_SIZES.iter() {
        if *c == key {
            log::debug!("[get_inventory_size] code={} source=hardcoded size=({},{})", key, w, h);
            return (*w, *h);
        }
    }
    // Category-based default: classify by item type from ALL_ITEMS
    // Standard D2R sizes: helm 2×2, gloves 2×2, boots 2×2, belt 1×2,
    // shield 2×2/2×3/2×4, armor 2×3/2×4, weapon 1×2/1×3/2×3/2×4
    for (c, _, is_a, is_w, is_s) in super::game_items::ALL_ITEMS.iter() {
        if *c != key { continue; }
        if *is_s { return (2, 2); }          // shield default
        if *is_a {
            let suffix: String = key.chars().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect();
            // Gloves: *gl, *lg, *vg, *mg, *tg
            if suffix.contains("gl") || key.ends_with("hgl") || key.ends_with("mgl") || key.ends_with("tgl") || key.ends_with("vgl") { return (2, 2); }
            // Boots: *bt, *bs, *hb, *vb
            if key.ends_with("bt") || key.ends_with("hb") || key.ends_with("vb") { return (2, 2); }
            // Belt: *bl, *lb, *hl, *lt
            if key.ends_with("bl") || key.ends_with("lb") || key.ends_with("hl") || key.ends_with("zhb") || key.ends_with("zlb") || key.ends_with("zmb") || key.ends_with("ztb") { return (1, 2); }
            // Body armor (longer code = later tier, usually 2×4)
            if key.len() >= 3 && matches!(key.as_bytes()[0], b'u' | b'x' | b'a') { return (2, 4); }
            if key.len() == 3 { let c = key.as_bytes()[0]; if c > b'g' && c <= b'u' { return (2, 4); } }
            return (2, 3); // other armor default
        }
        if *is_w {
            // 2H weapons usually have wider codes (2ax, 2hs, etc.)
            if key.len() == 3 { let c = key.as_bytes()[0]; if c == b'2' || c == b'7' || c == b'8' || c == b'9' { return (2, 3); } }
            if key.len() == 3 && key.as_bytes()[0] == b'6' { return (2, 4); }
            return (1, 2); // 1H weapon default
        }
        return (1, 1);
    }
    (1, 1) // ultimate fallback
}

/// Check if a code is stackable
pub fn is_stackable(code: &str) -> bool {
    let key = code.trim().to_lowercase();
    // Runtime (TXT) first
    if let Some(data) = ITEM_DATA.get()
        && data.stackable_set.contains(&key) {
            return true;
        }
    // Fallback to hardcoded
    super::constants::STACKABLE_ITEM_CODES.contains(&key.as_str())
}

/// Check if a code is mod-added (not in vanilla ALL_ITEMS)
pub fn is_mod_item(code: &str) -> bool {
    let key = code.trim().to_lowercase();
    if let Some(data) = ITEM_DATA.get() {
        return data.mod_codes.contains(&key);
    }
    // No runtime data: diff against ALL_ITEMS on-the-fly
    !ALL_ITEMS.iter().any(|(c, _, _, _, _)| *c == key)
}

/// Get the full ItemDef for a code (runtime only, None if not loaded or not found)
pub fn get_item_def(code: &str) -> Option<&'static ItemDef> {
    let key = code.trim().to_lowercase();
    ITEM_DATA.get()?.items.get(&key)
}

/// Read a TXT file (TSV format) and return rows as key-value maps.
/// Duplicated from item_names.rs to avoid circular deps.
fn read_txt_file(path: &Path) -> Option<Vec<HashMap<String, String>>> {
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    #[allow(clippy::lines_filter_map_ok)]
    let mut lines = reader.lines().flatten();

    let header = lines.next()?;
    let headers: Vec<&str> = header.split('\t').collect();
    if headers.len() < 2 {
        return None;
    }

    let rows: Vec<HashMap<String, String>> = lines
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < headers.len() {
                return None;
            }
            let mut row = HashMap::new();
            for (i, h) in headers.iter().enumerate() {
                let val = fields[i].trim();
                if !val.is_empty() {
                    row.insert(h.to_string(), val.to_string());
                }
            }
            Some(row)
        })
        .collect();

    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_loaded_returns_fallback() {
        // When not initialized, should return from hardcoded item_sizes
        // cap (Cap) has correct size 2×2 from game data, not the old wrong 1×1
        let size = get_inventory_size("cap");
        assert_eq!(size, (2, 2));
    }

    #[test]
    fn test_mod_detection_without_load() {
        // "r01" is vanilla — should be false
        assert!(!is_mod_item("r01"));
        // "zzz" is not in ALL_ITEMS — should be true
        assert!(is_mod_item("zzz"));
    }

    #[test]
    fn test_stackable_without_load() {
        assert!(is_stackable("r01"));
        assert!(!is_stackable("cap"));
    }

    #[test]
    fn test_grimoire_offhand_family() {
        assert!(is_grimoire_offhand("wa1"));
        assert!(is_grimoire_offhand("wae"));
        assert!(is_grimoire_offhand("WAF"));
        assert_eq!(get_inventory_size("wae"), (2, 2));
        assert!(!is_grimoire_offhand("uap"));
    }
}
