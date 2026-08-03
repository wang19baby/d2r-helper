//! ⚠️ DEPRECATED: Use `resource::NameResolver` instead.
//! 此文件仅在 profile_id=0（数据库未初始化）时作为 fallback 保留。

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use super::game_items::ALL_ITEMS;

// ── 内联常量（原 chinese_item_names.rs / game_item_names.rs）──

/// 中文物品名覆盖（33 条来自 D2R localization JSON）
const CHINESE_ITEM_NAMES: &[(&str, &str)] = &[
  ("7fl", "scourge"), ("9ax", "cleaver"), ("9cl", "cudgel"),
  ("9ha", "小斧"), ("9mp", "鸦嘴锄"), ("9p9", "lance"),
  ("bld", "利刃"), ("ci0", "circlet"), ("ci3", "权冠"),
  ("eyz", "eye"), ("fng", "fang"), ("gld", "Gold"),
  ("gsb", "Sapphire"), ("gsg", "Emerald"), ("gsr", "Ruby"),
  ("gsw", "Diamond"), ("hrn", "horn"), ("hrt", "heart"),
  ("jew", "珠宝"), ("key", "钥匙"), ("lbt", "靴子"),
  ("mau", "锤击"), ("msk", "mask"), ("qll", "quill"),
  ("scy", "scythe"), ("sku", "Skull"), ("sol", "Soul"),
  ("spr", "矛"), ("tch", "火炬"), ("uow", "aegis"),
  ("uts", "ward"), ("wnd", "wand"), ("xlm", "casque"),
];

/// 仙道轮回 mod 自定义物品占位符
const MOD_ITEM_NAMES: &[(&str, &str)] = &[
  ("w  s", ""), ("h  i", ""), ("u  i", ""), ("2  s", ""),
  ("u  s", ""), ("ctxc", ""), ("c7tm", ""), ("sbxh", ""),
  ("3 h", ""), ("ia7d", ""), ("gd", ""),
];

/// Build item name map from game data directory.
/// `language` selects which field to use from JSON strings (e.g., "zhCN", "zhTW", "enUS")
/// ⚠️ DEPRECATED: Use `resource::NameResolver` instead.
pub fn build_name_map_from_path(data_path: &str, language: &str) -> Option<HashMap<String, String>> {
    let dir = Path::new(data_path);
    if !dir.exists() || !dir.is_dir() {
        return None;
    }

    // Step 1: Read TXT files → code → English name
    let mut code_to_en: HashMap<String, String> = HashMap::new();
    for filename in &["misc.txt", "armor.txt", "weapons.txt"] {
        let p = dir.join("base").join(filename);
        if !p.exists() { let p = dir.join(filename); if !p.exists() { continue; } else { load_txt_codes(&p, &mut code_to_en); } }
        else { load_txt_codes(&p, &mut code_to_en); }
    }

    // Step 2: Read JSON localization files → build two maps:
    //   a) en_to_zh: English Key → localized name (for TXT code matching)
    //   b) code_to_zh: direct item code → localized name (for generated codes like lin, lsh)
    let mut en_to_zh: HashMap<String, String> = HashMap::new();
    let mut code_to_zh: HashMap<String, String> = HashMap::new();
    if let Some(mod_root) = find_mod_root(data_path) {
        let lng_dir = mod_root.join("data").join("local").join("lng").join("strings");
        for json_file in &["item-names.json", "item-runes.json", "item-gems.json", "item-nameaffixes.json", "item-rarenames.json", "item-modifiers.json"] {
            load_json_strings(&lng_dir.join(json_file), &mut en_to_zh, language);
        }
        // Also check strings-legacy for entries that might be missing
        let legacy_dir = mod_root.join("data").join("local").join("lng").join("strings-legacy");
        if legacy_dir.is_dir() {
            for json_file in &["item-names.json", "item-runes.json", "item-gems.json"] {
                load_json_strings(&legacy_dir.join(json_file), &mut en_to_zh, language);
            }
        }
        // Build direct code→zhCN map: Keys that look like item codes (3-4 ascii chars)
        for (key, name) in &en_to_zh {
            if key.len() <= 4 && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c.is_ascii_uppercase()) {
                code_to_zh.insert(key.clone(), name.clone());
            }
        }
    }

    // Step 3: Combine: TXT code → localized name, PLUS direct code mapping
    let mut map: HashMap<String, String> = HashMap::new();
    for (code, en_name) in &code_to_en {
        let zh = en_to_zh.get(&en_name.to_lowercase());
        map.insert(code.clone(), zh.cloned().unwrap_or_else(|| en_name.clone()));
    }
    // Add direct code mappings (overrides TXT-based entries where applicable)
    for (code, name) in &code_to_zh {
        map.insert(code.clone(), name.clone());
    }

    Some(map)
}

/// Find the mod root directory from the excel path.
/// Returns the mod root (parent of data/) so callers can do mod_root/data/...
fn find_mod_root(data_path: &str) -> Option<std::path::PathBuf> {
    let p = Path::new(data_path);
    for ancestor in p.ancestors() {
        // Case 1: ancestor has local/lng/strings → ancestor includes data/
        if ancestor.join("local").join("lng").join("strings").is_dir() {
            // Return the parent of ancestor (which includes data/)
            return ancestor.parent().map(|p| p.to_path_buf());
        }
        // Case 2: ancestor has data/local/lng/strings → ancestor is mod root
        if ancestor.join("data").join("local").join("lng").join("strings").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Read TXT file and populate code→name map
fn load_txt_codes(path: &Path, map: &mut HashMap<String, String>) {
    if let Some(rows) = read_txt_file(path) {
        for row in rows {
            if let Some(code) = row.get("code")
                && let Some(name) = row.get("name") {
                    map.insert(code.trim().to_lowercase(), name.to_string());
                }
        }
    }
}

/// Read JSON string file and populate English→localized name map
fn load_json_strings(path: &Path, map: &mut HashMap<String, String>, language: &str) {
    if !path.exists() { return; }
    let file = match std::fs::File::open(path) { Ok(f) => f, Err(_) => return };
    let mut raw = String::new();
    use std::io::Read;
    if std::io::BufReader::new(file).read_to_string(&mut raw).is_err() { return; }
    if raw.starts_with('\u{FEFF}') { raw = raw[3..].to_string(); }
    // D2RMM mod adds // comments to JSON files before the array.
    // Strip them: [//comment\n//comment\n{...}] → [{...}]
    let cleaned: Vec<String> = raw.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { String::new() }  // full-line comment
            else if t.starts_with("[//") { "[".to_string() }  // [//comment
            else { l.to_string() }
        })
        .filter(|l| !l.is_empty())
        .collect();
    let raw = if cleaned.is_empty() { raw } else { cleaned.join("\n") };
    let raw = raw.replace('\r', "");

    let entries: Vec<serde_json::Value> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => { eprintln!("[load_json] JSON error in {:?}: {} (raw[..100]={:?})", path, e, &raw[..100.min(raw.len())]); return; }
    };
    for entry in entries {
        let key = entry.get("Key").and_then(|v| v.as_str()).map(|s| s.to_lowercase());
        // Try requested language first, then fall back to English
        let name = entry.get(language).or_else(|| entry.get("enUS")).and_then(|v| v.as_str());
        if let (Some(k), Some(n)) = (key, name) {
            // Take the last non-empty line (D2RMM entries have affix on line 1, name on line 2)
            let clean_base = n.split('\n').rfind(|l| !l.trim().is_empty())
                .unwrap_or(n)
                .trim()
                .to_string();
            // Strip D2R color codes: ÿcX where X is the color specifier
            // ÿ = 2 bytes UTF-8, c = 1 byte, specifier = 1 byte = 4 total
            let mut clean = clean_base;
            for _ in 0..50 {
                let bytes = clean.as_bytes();
                let mut found = false;
                for i in 0..bytes.len().saturating_sub(2) {
                    if bytes[i] == 0xC3 && bytes[i+1] == 0xBF && bytes[i+2] == b'c' {
                        // Remove ÿc + color spec (4 bytes total: ÿ=2, c=1, X=1)
                        let remove = (i + 4).min(bytes.len());
                        clean = String::from_utf8_lossy(&bytes[..i]).to_string() +
                                &String::from_utf8_lossy(&bytes[remove..]);
                        found = true;
                        break;
                    }
                }
                if !found { break; }
            }
            let clean = clean.trim().to_string();
            if !clean.is_empty() {
                map.insert(k, clean.to_string());
            }
        }
    }
}

/// Read a TXT file (TSV format) and return rows as key-value maps
fn read_txt_file(path: &Path) -> Option<Vec<HashMap<String, String>>> {
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    #[allow(clippy::lines_filter_map_ok)]
    let mut lines = reader.lines().flatten();

    let header = lines.next()?;
    let headers: Vec<&str> = header.split('\t').collect();

    let rows: Vec<HashMap<String, String>> = lines
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < headers.len() { return None; }
            let mut row = HashMap::new();
            for (i, h) in headers.iter().enumerate() {
                if !fields[i].is_empty() {
                    row.insert(h.to_string(), fields[i].to_string());
                }
            }
            Some(row)
        })
        .collect();

    Some(rows)
}

/// 构造英文基础名表（不带任何中文覆盖）。
/// ⚠️ 仅作为旧 fallback 保留。新代码请用 NameResolver。
fn build_english_name_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (code, name, _is_a, _is_w, _is_s) in ALL_ITEMS {
        map.insert(code.to_string(), name.to_string());
    }
    map
}

/// Resolve an item's display name from code.
/// ⚠️ DEPRECATED: Use `resource::NameResolver` instead.
pub fn build_full_name_map() -> HashMap<String, String> {
    let mut map = build_english_name_map();

    // 1. Chinese names (from localization files) — override when available
    for (code, name) in CHINESE_ITEM_NAMES {
        map.insert(code.to_string(), name.to_string());
    }

    // 2. User-maintained mod items mapping (MOD_ITEM_NAMES in chinese_item_names.rs)
    for (code, name) in MOD_ITEM_NAMES {
        if !name.is_empty() {
            map.insert(code.to_string(), name.to_string());
        }
    }

    map
}

/// 构造内置名称表。
///
/// - `enUS`：纯英文名
/// - `zhCN`：简体中文（若内置缺失，回落英文）
/// - `zhTW`：当前仓库没有独立繁体内置表，先回落到简体中文
fn build_embedded_name_map(language: &str) -> HashMap<String, String> {
    match language {
        "zhCN" => build_full_name_map(),
        "zhTW" => build_full_name_map(),
        _ => build_english_name_map(),
    }
}

/// Try to load localized names from game data path. Falls back to embedded defaults.
pub fn load_item_names(game_data_path: Option<&str>, language: &str) -> HashMap<String, String> {
    if let Some(path) = game_data_path
        && let Some(map) = build_name_map_from_path(path, language) {
            return map;
        }
    build_embedded_name_map(language)
}

/// Resolve an item's display name from code.
/// Falls back to a human-readable generated name for magic/rare items.
pub fn resolve_item_name(code: &str, quality: Option<u8>, map: &HashMap<String, String>) -> String {
    if let Some(name) = map.get(code) {
        return name.clone();
    }

    // 第二优先级: 用户手动维护的 mod items 映射
    // 用户在 chinese_item_names.rs::MOD_ITEM_NAMES 中填入 mod 装备中文名
    for (known_code, name) in MOD_ITEM_NAMES {
        if *known_code == code && !name.is_empty() {
            return name.to_string();
        }
    }

    // For items with generated codes (charms, jewels, rings), use quality prefix
    let prefix = match quality {
        Some(7) => "Unique ",
        Some(6) => "Rare ",
        Some(5) => "Set ",
        Some(4) => "Magic ",
        _ => "",
    };

    let c = code.to_lowercase();
    let in_vanilla = super::game_items::ALL_ITEMS.iter().any(|(known, _, _, _, _)| *known == c);

    // Step 1: 如果是已知 vanilla item,返回 code(vanilla item 不需要 Mod: 前缀)
    if in_vanilla {
        return format!("{}{}", prefix, c);
    }

    // Step 2: 根据首字母推测类别 (用于生成物品 charm/jewel/ring 等)
    let base: String = match c.as_bytes().first() {
        Some(b'l') if c.len() == 3 => {
            match &c[1..] {
                "sc" | "s " => "Small Charm".to_string(),
                "gc" | "g " => "Grand Charm".to_string(),
                "tc" | "t " => "Large Charm".to_string(),
                _ => "Charm".to_string(),
            }
        },
        Some(b'j') => "Jewel".to_string(),
        Some(b'n') => "Ring/Amulet".to_string(),
        Some(b'b') => "Body Part".to_string(),
        Some(b'd') => "Dye".to_string(),
        // 'u'/'x'/'m'/'c' 是 mod items 的常见首字母,不再使用"Elite Item"/"Class Item" 兜底
        _ => format!("Mod:{}", code),  // 真正的 mod item 标记
    };

    format!("{}{}", prefix, base)
}

#[cfg(test)]
mod mod_item_fallback_tests {
    use super::*;

    #[test]
    fn test_mod_items_fallback_to_mod_prefix() {
        // 11 个仙道轮回 mod 自定义 code 在 ALL_ITEMS 和 name_map 中都不存在
        let mod_codes = ["w  s", "h  i", "u  i", "2  s", "u  s",
                         "ctxc", "c7tm", "sbxh", "3 h", "ia7d", "gd"];

        for code in &mod_codes {
            let name = resolve_item_name(code, Some(7), &HashMap::new());
            // 必须以 "Unique Mod:" 开头 (quality=7 + mod prefix)
            assert!(name.starts_with("Unique Mod:"),
                "mod code '{}' should fall back to 'Unique Mod:{}' but got '{}'",
                code, code, name);
        }
    }

    #[test]
    fn test_quality_prefix_applied_to_mod() {
        // Unique mod item: 应有 "Unique Mod:xxx" 形式
        let name = resolve_item_name("w  s", Some(7), &HashMap::new());
        assert_eq!(name, "Unique Mod:w  s");

        // Rare mod item
        let name = resolve_item_name("ctxc", Some(6), &HashMap::new());
        assert_eq!(name, "Rare Mod:ctxc");

        // Normal quality mod item: 无前缀
        let name = resolve_item_name("w  s", Some(2), &HashMap::new());
        assert_eq!(name, "Mod:w  s");
    }

    #[test]
    fn test_known_vanilla_codes_not_mod_prefix() {
        // 标准 vanilla 物品不应被加 Mod: 前缀
        // cap 是 vanilla armor → 保持 "cap"
        let name = resolve_item_name("cap", Some(2), &HashMap::new());
        assert_eq!(name, "cap");

        // rin 是 vanilla ring → "Unique rin" (在 ALL_ITEMS 中,直接返回)
        let name = resolve_item_name("rin", Some(7), &HashMap::new());
        assert_eq!(name, "Unique rin");

        // lsc 是 vanilla small charm → "Magic lsc"
        let name = resolve_item_name("lsc", Some(4), &HashMap::new());
        assert_eq!(name, "Magic lsc");
    }

    #[test]
    fn test_full_name_map_includes_vanilla_items() {
        // build_full_name_map 应该包含 vanilla items
        let map = build_full_name_map();
        // 验证一些已知 vanilla items
        assert!(map.contains_key("cap"), "vanilla cap should be in full name map");
        assert!(map.contains_key("r01"), "vanilla rune should be in full name map");
    }

    #[test]
    fn test_mod_item_names_placeholder_exists() {
        // 验证 MOD_ITEM_NAMES 存在且可被查询(即使所有 entry 都是 placeholder)
        use super::MOD_ITEM_NAMES;
        let total = MOD_ITEM_NAMES.len();
        // 至少应该有 11 个 placeholder entry
        assert!(total >= 11, "MOD_ITEM_NAMES should have at least 11 placeholder entries for Page[0] mod items, got {}", total);
    }

    #[test]
    fn test_resolve_with_explicit_name_map() {
        // 当 name_map 显式包含时,优先使用
        let mut map = HashMap::new();
        map.insert("w  s".to_string(), "战争之剑".to_string());
        let name = resolve_item_name("w  s", Some(7), &map);
        assert_eq!(name, "战争之剑", "explicit map should win over fallback");
    }
}
