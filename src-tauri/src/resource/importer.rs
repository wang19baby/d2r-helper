//! TXT/JSON structured data importer.
//! 从磁盘读取 D2R 游戏 TXT 文件（misc.txt, armor.txt, weapons.txt,
//! uniqueitems.txt, setitems.txt, sets.txt），解析后写入 SQLite 定义表。
//! 当磁盘 TXT 不可用时，退化到从硬编码 Rust 常量导入（game_items.rs 等）。

use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// TXT file parsing result: rows as key-value maps.
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
                if !fields[i].is_empty() {
                    row.insert(h.to_string(), fields[i].to_string());
                }
            }
            Some(row)
        })
        .collect();

    if rows.is_empty() {
        None
    } else {
        Some(rows)
    }
}

/// Resolve the actual path for a TXT filename in the excel directory.
/// Priority: `<excel_path>/<filename>` first (mod override), then `base/<filename>` (vanilla fallback).
fn resolve_txt_path(excel_path: &str, filename: &str) -> Option<PathBuf> {
    let dir = Path::new(excel_path);
    let root_path = dir.join(filename);
    if root_path.exists() {
        return Some(root_path);
    }
    let base_path = dir.join("base").join(filename);
    if base_path.exists() {
        return Some(base_path);
    }
    None
}

/// Resolve the main (non-base) path for a TXT file.
fn resolve_main_txt_path(excel_path: &str, filename: &str) -> Option<PathBuf> {
    let dir = Path::new(excel_path);
    let root_path = dir.join(filename);
    if root_path.exists() { Some(root_path) } else { None }
}

/// ResourceImporter: imports TXT game data into SQLite definition tables.
pub struct ResourceImporter<'a> {
    conn: &'a Connection,
    profile_id: i64,
}

impl<'a> ResourceImporter<'a> {
    pub fn new(conn: &'a Connection, profile_id: i64) -> Self {
        Self { conn, profile_id }
    }

    /// Run all available imports in dependency order.
    /// Safe to call multiple times — uses INSERT OR REPLACE.
    pub fn import_all(&self, excel_path: &str) -> Vec<ImportResult> {
        let mut results = Vec::new();
        results.push(self.import_item_base(excel_path));
        results.push(self.import_unique_items(excel_path));
        results.push(self.import_set_items(excel_path));
        results.push(self.import_runewords(excel_path));
        results.push(self.import_stat_def(excel_path));
        results.push(self.import_skill_def(excel_path));
        results.push(self.import_item_affix(excel_path));
        results.push(self.import_item_types());
        results
    }

    /// Import item_base from misc.txt, armor.txt, weapons.txt.
    /// Falls back to hardcoded ALL_ITEMS if TXT unavailable.
    pub fn import_item_base(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();

        // Try TXT files first
        let mut txt_rows: Vec<HashMap<String, String>> = Vec::new();
        for filename in &["misc.txt", "armor.txt", "weapons.txt"] {
            if let Some(path) = resolve_txt_path(excel_path, filename)
                && let Some(rows) = read_txt_file(&path) {
                    let category = filename.replace(".txt", "");
                    for mut row in rows {
                        row.entry("_category".to_string()).or_insert_with(|| category.clone());
                        txt_rows.push(row);
                    }
                }
        }

        let count: usize;
        let source: &str;

        if !txt_rows.is_empty() {
            source = "misc.txt,armor.txt,weapons.txt";
            count = self._import_item_base_from_rows(&txt_rows);
        } else {
            // Fallback: import from hardcoded constants
            source = "const";
            count = self._import_item_base_from_constants();
        }

        let elapsed = start.elapsed();
        self._log_import("item_base", count as i64, source, elapsed);

        ImportResult {
            table: "item_base".to_string(),
            rows: count,
            source: source.to_string(),
            elapsed,
        }
    }

    /// Import unique_item_def from uniqueitems.txt.
    /// Falls back to hardcoded UNIQUE_ITEMS constant.
    pub fn import_unique_items(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let count: usize;
        let source: &str;

        if let Some(path) = resolve_txt_path(excel_path, "uniqueitems.txt") {
            if let Some(rows) = read_txt_file(&path) {
                source = "uniqueitems.txt";
                count = self._import_unique_items_from_txt(&rows);
            } else {
                source = "const";
                count = self._import_unique_items_from_constants();
            }
        } else {
            source = "const";
            count = self._import_unique_items_from_constants();
        }

        let elapsed = start.elapsed();
        self._log_import("unique_item_def", count as i64, source, elapsed);

        ImportResult {
            table: "unique_item_def".to_string(),
            rows: count,
            source: source.to_string(),
            elapsed,
        }
    }

    /// Import set_def + set_item_def from setitems.txt and sets.txt.
    /// Falls back to hardcoded SET_ITEMS + SET_BONUSES constants.
    pub fn import_set_items(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let count: usize;
        let source: &str;

        // Try to get data from TXT files
        let setitems_path = resolve_txt_path(excel_path, "setitems.txt");
        let sets_path = resolve_txt_path(excel_path, "sets.txt");

        if let (Some(sp), Some(smp)) = (&setitems_path, &sets_path) {
            if let (Some(setitems_rows), Some(sets_rows)) = (read_txt_file(sp), read_txt_file(smp)) {
                source = "setitems.txt,sets.txt";
                count = self._import_set_items_from_txt(&setitems_rows, &sets_rows);
            } else {
                source = "const";
                count = self._import_set_items_from_constants();
            }
        } else {
            source = "const";
            count = self._import_set_items_from_constants();
        }

        let elapsed = start.elapsed();
        self._log_import("set_item_def", count as i64, source, elapsed);

        ImportResult {
            table: "set_item_def".to_string(),
            rows: count,
            source: source.to_string(),
            elapsed,
        }
    }

    // ── Private: item_base from TXT rows ──

    fn _import_item_base_from_rows(&self, rows: &[HashMap<String, String>]) -> usize {
        let vanilla_codes: std::collections::HashSet<&str> =
            crate::protocol::d2i::legacy::game_items::ALL_ITEMS
                .iter()
                .map(|(c, _, _, _, _)| *c)
                .collect();

        let mut count = 0usize;
        for row in rows {
            let code = match row.get("code") {
                Some(c) => c.trim().to_lowercase(),
                None => continue,
            };
            if code.is_empty() {
                continue;
            }

            let name_en = row.get("name").map(|s| s.to_string()).unwrap_or_default();
            let item_type = row.get("type").or_else(|| row.get("*type")).map(|s| s.to_string()).unwrap_or_default();
            let category = row.get("_category").map(|s| s.to_string()).unwrap_or_default();
            let inv_width = row.get("invwidth").and_then(|v| v.parse::<u8>().ok()).unwrap_or(1);
            let inv_height = row.get("invheight").and_then(|v| v.parse::<u8>().ok()).unwrap_or(1);
            let stackable = row.get("stackable").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) != 0;
            let has_inv = row.get("hasinv").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) != 0;
            let level = row.get("level").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let level_req = row.get("levelreq").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let gem_apply = row.get("gemapply").or_else(|| row.get("gemsockets")).map(|s| s.to_string()).unwrap_or_default();

            let is_grimoire = crate::protocol::d2i::legacy::game_data_loader::is_grimoire_offhand(&code);
            let is_armor = !is_grimoire && category == "armor";
            let is_weapon = !is_grimoire && category == "weapons";
            let is_shield = is_grimoire
                || row.get("type").map(|t| t == "shie" || t == "ashd").unwrap_or(false);
            let item_category = if is_grimoire {
                "offhand"
            } else {
                category.as_str()
            };
            let is_mod_item = !vanilla_codes.contains(code.as_str());

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO item_base
                    (code, profile_id, name_en, item_type, item_category,
                     inv_width, inv_height, stackable, has_inventory,
                     level, level_req, gem_apply, is_armor, is_weapon, is_shield, is_mod_item)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    code, self.profile_id, name_en, item_type, item_category,
                    inv_width, inv_height, stackable as i32, has_inv as i32,
                    level, level_req, gem_apply, is_armor as i32, is_weapon as i32, is_shield as i32, is_mod_item as i32,
                ],
            ) {
                log::warn!("[importer] item_base insert error for {}: {}", code, e);
                continue;
            }
            count += 1;
        }
        count
    }

    /// Fallback: import item_base from hardcoded ALL_ITEMS constant.
    fn _import_item_base_from_constants(&self) -> usize {
        let mut count = 0usize;

        for (code, name_en, is_armor, is_weapon, is_shield) in
            crate::protocol::d2i::legacy::game_items::ALL_ITEMS.iter()
        {
            let name_en = name_en.to_string();
            let is_grimoire = crate::protocol::d2i::legacy::game_data_loader::is_grimoire_offhand(code);
            let item_category = if is_grimoire {
                "offhand"
            } else if *is_armor {
                "armor"
            } else if *is_weapon {
                "weapons"
            } else {
                "misc"
            };
            let stored_is_armor = *is_armor && !is_grimoire;
            let stored_is_weapon = *is_weapon && !is_grimoire;
            let stored_is_shield = *is_shield || is_grimoire;

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO item_base
                    (code, profile_id, name_en, item_type, item_category,
                     inv_width, inv_height, stackable, has_inventory,
                     level, level_req, gem_apply, is_armor, is_weapon, is_shield, is_mod_item)
                 VALUES (?1, ?2, ?3, '', ?4, 1, 1, 0, 0, 0, 0, '', ?5, ?6, ?7, 0)",
                params![
                    code, self.profile_id, name_en, item_category,
                    stored_is_armor as i32, stored_is_weapon as i32, stored_is_shield as i32,
                ],
            ) {
                log::warn!("[importer] item_base const-insert error for {}: {}", code, e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Private: unique items from TXT ──

    fn _import_unique_items_from_txt(&self, rows: &[HashMap<String, String>]) -> usize {
        let mut count = 0usize;
        for row in rows {
            // D2R: *ID=number, index=name
            let unique_id: u16 = match row.get("*ID").and_then(|v| v.parse().ok()) {
                Some(id) => id,
                None => continue,
            };
            // D2R: index=name, *ID=number
            let name_en = row.get("index").map(|s| s.to_string()).unwrap_or_default();
            let base_code = row.get("code").map(|s| s.trim().to_lowercase()).unwrap_or_default();
            let level = row.get("lvl").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let level_req = row.get("lvl req").or_else(|| row.get("levelreq")).and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO unique_item_def
                    (profile_id, unique_id, name_en, base_code, level, level_req, is_mod_item)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                params![self.profile_id, unique_id, name_en, base_code, level, level_req],
            ) {
                log::warn!("[importer] unique_item_def insert error for {}: {}", unique_id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    fn _import_unique_items_from_constants(&self) -> usize {
        let mut count = 0usize;
        for (id, name, code, level, level_req, is_mod) in
            crate::protocol::d2i::legacy::unique_items::UNIQUE_ITEMS.iter()
        {
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO unique_item_def
                    (profile_id, unique_id, name_en, base_code, level, level_req, is_mod_item)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![self.profile_id, id, name, code, level, level_req, *is_mod as i32],
            ) {
                log::warn!("[importer] unique_item_def const-insert error for {}: {}", id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Private: set items from TXT ──

    fn _import_set_items_from_txt(
        &self,
        setitems_rows: &[HashMap<String, String>],
        sets_rows: &[HashMap<String, String>],
    ) -> usize {
        // Step 1: Import set_def from sets.txt
        // D2R: index=name, no *ID column — assign sequential IDs
        let mut set_name_to_id: std::collections::HashMap<String, u16> = std::collections::HashMap::new();
        for (idx, row) in sets_rows.iter().enumerate() {
            let set_id = idx as u16;
            let name_en = row.get("index").or_else(|| row.get("name")).map(|s| s.to_string()).unwrap_or_default();
            if name_en.is_empty() { continue; }
            set_name_to_id.insert(name_en.clone(), set_id);
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO set_def (profile_id, set_id, name_en) VALUES (?1, ?2, ?3)",
                params![self.profile_id, set_id, name_en],
            ) {
                log::warn!("[importer] set_def insert error for {}: {}", set_id, e);
            }
        }

        // Step 1b: Import set_bonus_def from sets.txt PCode/PMin/PMax columns
        // 多件套加成: PCode2a..5b = 2件/3件/4件/5件套; 全套加成: FCode1..8
        let mut bonus_count = 0usize;
        for (idx, row) in sets_rows.iter().enumerate() {
            let set_id = idx as u16;
            // 多件套 (piece_count 2-5)
            for pieces in 2..=5u8 {
                for suffix in ["a", "b"] {
                    let code = row.get(&format!("PCode{}{}", pieces, suffix))
                        .map(|s| s.trim().to_string()).unwrap_or_default();
                    if code.is_empty() { continue; }
                    let stat_id = match crate::protocol::d2s::magic_affix::prop_code_to_stat_id(&code) {
                        Some(id) => id,
                        None => { log::warn!("[importer] set bonus unknown prop code '{}' (set={} pieces={}{})", code, set_id, pieces, suffix); continue; }
                    };
                    let param = row.get(&format!("PParam{}{}", pieces, suffix))
                        .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
                    let min_v = row.get(&format!("PMin{}{}", pieces, suffix))
                        .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
                    let max_v = row.get(&format!("PMax{}{}", pieces, suffix))
                        .and_then(|v| v.parse::<i32>().ok()).unwrap_or(min_v);
                    if let Err(e) = self.conn.execute(
                        "INSERT OR REPLACE INTO set_bonus_def
                            (profile_id, set_id, piece_count, stat_id, param, min_value, max_value)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![self.profile_id, set_id, pieces, stat_id, param, min_v, max_v],
                    ) {
                        log::warn!("[importer] set_bonus_def insert error: {}", e);
                        continue;
                    }
                    bonus_count += 1;
                }
            }
            // 全套加成 (FCode1..8, piece_count=6)
            for i in 1..=8u8 {
                let code = row.get(&format!("FCode{}", i))
                    .map(|s| s.trim().to_string()).unwrap_or_default();
                if code.is_empty() { continue; }
                let stat_id = match crate::protocol::d2s::magic_affix::prop_code_to_stat_id(&code) {
                    Some(id) => id,
                    None => { log::warn!("[importer] set full bonus unknown prop code '{}' (set={})", code, set_id); continue; }
                };
                let param = row.get(&format!("FParam{}", i))
                    .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
                let min_v = row.get(&format!("FMin{}", i))
                    .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
                let max_v = row.get(&format!("FMax{}", i))
                    .and_then(|v| v.parse::<i32>().ok()).unwrap_or(min_v);
                if let Err(e) = self.conn.execute(
                    "INSERT OR REPLACE INTO set_bonus_def
                        (profile_id, set_id, piece_count, stat_id, param, min_value, max_value)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![self.profile_id, set_id, 6u8, stat_id, param, min_v, max_v],
                ) {
                    log::warn!("[importer] set_bonus_def full insert error: {}", e);
                    continue;
                }
                bonus_count += 1;
            }
        }
        if bonus_count > 0 {
            self._log_import("set_bonus_def", bonus_count as i64, "sets.txt", std::time::Duration::ZERO);
        }

        // Step 3: Import set_item_def from setitems.txt
        let mut count = 0usize;
        for row in setitems_rows {
            // D2R: *ID=number, index=name, item=base_code
            let item_id: u16 = match row.get("*ID").and_then(|v| v.parse().ok()) {
                Some(id) => id,
                None => continue,
            };
            // D2R: "set" column contains the SET NAME, look up numeric ID
            let set_id: u16 = row.get("set")
                .and_then(|name| set_name_to_id.get(name).copied())
                .unwrap_or(0);
            let name_en = row.get("index").map(|s| s.to_string()).unwrap_or_default();
            let base_code = row.get("item").or_else(|| row.get("code")).map(|s| s.trim().to_lowercase()).unwrap_or_default();
            let level = row.get("lvl").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let level_req = row.get("lvl req").or_else(|| row.get("levelreq")).and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO set_item_def
                    (profile_id, set_id, item_id, name_en, base_code, level, level_req)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![self.profile_id, set_id, item_id, name_en, base_code, level, level_req],
            ) {
                log::warn!("[importer] set_item_def insert error for {}: {}", item_id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    fn _import_set_items_from_constants(&self) -> usize {
        // Step 1: Import set_def from SET_BONUSES
        for (set_id, set_name) in crate::protocol::d2i::legacy::set_items::SET_BONUSES.iter().enumerate() {
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO set_def (profile_id, set_id, name_en) VALUES (?1, ?2, ?3)",
                params![self.profile_id, set_id, set_name],
            ) {
                log::warn!("[importer] set_def const-insert error for {}: {}", set_id, e);
            }
        }

        // Step 2: Import set_item_def from SET_ITEMS
        // We need to map set_name → set_id from SET_BONUSES
        let set_name_to_id: std::collections::HashMap<&str, usize> =
            crate::protocol::d2i::legacy::set_items::SET_BONUSES
                .iter()
                .enumerate()
                .map(|(i, n)| (*n, i))
                .collect();

        let mut count = 0usize;
        for (id, name, set_name, code, level, level_req) in
            crate::protocol::d2i::legacy::set_items::SET_ITEMS.iter()
        {
            let set_id = set_name_to_id.get(set_name).copied().unwrap_or(0) as u16;
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO set_item_def
                    (profile_id, set_id, item_id, name_en, base_code, level, level_req)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![self.profile_id, set_id, id, name, code, level, level_req],
            ) {
                log::warn!("[importer] set_item_def const-insert error for {}: {}", id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Phase 2.2: Runewords ──

    /// Import runeword_def from hardcoded RUNEWORDS constant.
    /// TXT fallback: reads runes.txt when available.
    pub fn import_runewords(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let count: usize;
        let source: &str;

        // Try TXT first (runes.txt)
        if let Some(path) = resolve_txt_path(excel_path, "runes.txt") {
            if let Some(rows) = read_txt_file(&path) {
                source = "runes.txt";
                count = self._import_runewords_from_txt(&rows);
            } else {
                source = "const";
                count = self._import_runewords_from_constants();
            }
        } else {
            source = "const";
            count = self._import_runewords_from_constants();
        }

        let elapsed = start.elapsed();
        self._log_import("runeword_def", count as i64, source, elapsed);
        ImportResult { table: "runeword_def".to_string(), rows: count, source: source.to_string(), elapsed }
    }

    fn _import_runewords_from_txt(&self, rows: &[HashMap<String, String>]) -> usize {
        let mut count = 0usize;
        for row in rows {
            // Cascview format: Name=runeword key, *Rune Name=display name, complete=1 means runeword
            // Older format: *runeword, name, rune1..rune6, *items, sockets
            let key = row.get("*runeword").or_else(|| row.get("runeword"))
                .or_else(|| row.get("Name")).or_else(|| row.get("name"))
                .map(|s| s.to_string()).unwrap_or_default();
            let name_en = row.get("name").or_else(|| row.get("*Rune Name"))
                .map(|s| s.to_string()).unwrap_or_default();

            // Skip non-runeword rows (individual runes have complete=empty)
            let complete = row.get("complete").map(|s| s.as_str()).unwrap_or("");
            if complete != "1" { continue; }

            // Rune codes: Rune1..Rune6 (cascview) or rune1..rune6 (old)
            let runes_raw = row.get("rune1").or_else(|| row.get("Rune1"))
                .map(|_| {
                    let mut codes = Vec::new();
                    for i in 1..=6 {
                        let col_lower = format!("rune{}", i);
                        let col_upper = format!("Rune{}", i);
                        if let Some(r) = row.get(&col_lower).or_else(|| row.get(&col_upper))
                            && !r.is_empty() { codes.push(r.clone()); }
                    }
                    codes.join(",")
                }).unwrap_or_default();

            // Base types: itype1..itype6 (cascview) or *items/items (old)
            let bases = row.get("*items").or_else(|| row.get("items"))
                .map(|s| s.to_string())
                .or_else(|| {
                    let mut types = Vec::new();
                    for i in 1..=6 {
                        let col = format!("itype{}", i);
                        if let Some(t) = row.get(&col)
                            && !t.is_empty() { types.push(t.clone()); }
                    }
                    if types.is_empty() { None }
                    else { Some(types.join(",")) }
                }).unwrap_or_default();

            // Socket count: explicit column or derived from rune count
            let sockets = row.get("sockets").and_then(|v| v.parse::<u8>().ok())
                .unwrap_or_else(|| {
                    if runes_raw.is_empty() { 0 }
                    else { runes_raw.split(',').count() as u8 }
                });

            if name_en.is_empty() { continue; }
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO runeword_def (profile_id, runeword_key, name_en, rune_codes, allowed_base_types, sockets)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![self.profile_id, key, name_en, runes_raw, bases, sockets],
            ) {
                log::warn!("[importer] runeword_def insert error: {}", e);
                continue;
            }
            count += 1;
        }
        count
    }

    fn _import_runewords_from_constants(&self) -> usize {
        let mut count = 0usize;
        for (key, name, runes, bases, sockets) in
            crate::protocol::d2i::legacy::runewords::RUNEWORDS.iter()
        {
            let rune_str = runes.join(",");
            let base_str = bases.join(",");
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO runeword_def (profile_id, runeword_key, name_en, rune_codes, allowed_base_types, sockets)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![self.profile_id, key, name, rune_str, base_str, sockets],
            ) {
                log::warn!("[importer] runeword_def const-insert error: {}", e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Phase 2.2: Stat Definitions ──

    /// Import stat_def from ItemStatCost.txt (TXT) or hardcoded MAGICAL_PROPS.
    pub fn import_stat_def(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let count: usize;
        let source: &str;

        if let Some(path) = resolve_txt_path(excel_path, "ItemStatCost.txt") {
            if let Some(rows) = read_txt_file(&path) {
                source = "ItemStatCost.txt";
                count = self._import_stat_def_from_txt(&rows);
            } else {
                source = "const";
                count = self._import_stat_def_from_constants();
            }
        } else {
            source = "const";
            count = self._import_stat_def_from_constants();
        }

        let elapsed = start.elapsed();
        self._log_import("stat_def", count as i64, source, elapsed);
        ImportResult { table: "stat_def".to_string(), rows: count, source: source.to_string(), elapsed }
    }

    fn _import_stat_def_from_txt(&self, rows: &[HashMap<String, String>]) -> usize {
        let mut count = 0usize;
        for row in rows {
            // D2R: *ID=number, Stat=name, "Save Bits" etc use spaced names
            let stat_id: u16 = match row.get("*ID").and_then(|v| v.parse().ok()) {
                Some(id) => id,
                None => continue,
            };
            let name_en = row.get("Stat").or_else(|| row.get("stat")).or_else(|| row.get("*stat"))
                .map(|s| s.to_string()).unwrap_or_default();
            let save_bits = row.get("Save Bits").or_else(|| row.get("savebits")).or_else(|| row.get("SaveBits"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let save_param_bits = row.get("Save Param Bits").or_else(|| row.get("saveparambits")).or_else(|| row.get("SaveParamBits"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let save_add = row.get("Save Add").or_else(|| row.get("saveadd")).or_else(|| row.get("SaveAdd"))
                .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
            let signed = row.get("Signed").or_else(|| row.get("signed"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let encoding = row.get("Encode").or_else(|| row.get("encode"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO stat_def (profile_id, stat_id, name_en, save_bits, save_param_bits, save_add, signed, encoding)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![self.profile_id, stat_id, name_en, save_bits, save_param_bits, save_add, signed, encoding],
            ) {
                log::warn!("[importer] stat_def insert error for {}: {}", stat_id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    fn _import_stat_def_from_constants(&self) -> usize {
        use crate::data::stat_cost::build_stat_table;
        let table = build_stat_table();
        let mut count = 0usize;
        for stat_id in 0..=419u16 {
            let prop = table.get(stat_id);
            if prop.save_bits == 0 && stat_id > 0 {
                continue; // skip empty/character-only entries
            }
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO stat_def (profile_id, stat_id, name_en, save_bits, save_param_bits, save_add, signed, encoding)
                 VALUES (?1, ?2, '', ?3, ?4, ?5, ?6, ?7)",
                params![self.profile_id, stat_id, prop.save_bits, prop.save_param_bits, prop.save_add, prop.signed, prop.encoding],
            ) {
                log::warn!("[importer] stat_def const-insert error for {}: {}", stat_id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Phase 2.2: Skill Definitions ──

    /// Import skill_def + skill_tab_def from skills.txt and skilldesc.txt.
    /// No hardcoded fallback — silently skipped when TXT files unavailable.
    pub fn import_skill_def(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let mut count: usize;
        let source: &str;

        let skills_path = resolve_txt_path(excel_path, "skills.txt");
        let skilldesc_path = resolve_txt_path(excel_path, "skilldesc.txt");

        if let (Some(sp), Some(sdp)) = (&skills_path, &skilldesc_path) {
            if let (Some(skill_rows), Some(_desc_rows)) = (read_txt_file(sp), read_txt_file(sdp)) {
                source = "skills.txt";
                count = self._import_skill_def_from_txt(&skill_rows, excel_path);
                // base/skills.txt 可能不全（如只有 0-372），再导入主目录版本补齐
                if let Some(main_path) = resolve_main_txt_path(excel_path, "skills.txt")
                    && main_path != *sp
                        && let Some(main_rows) = read_txt_file(&main_path) {
                            let extra = self._import_skill_def_from_txt(&main_rows, excel_path);
                            count = count.max(extra);
                            log::info!("[importer] skill_def extra rows from main path: {}", extra);
                        }
            } else {
                source = "none";
                count = 0;
            }
        } else {
            source = "none";
            count = 0;
        }

        let elapsed = start.elapsed();
        self._log_import("skill_def", count as i64, source, elapsed);
        ImportResult { table: "skill_def".to_string(), rows: count, source: source.to_string(), elapsed }
    }

    fn _import_skill_def_from_txt(&self, rows: &[HashMap<String, String>], _excel_path: &str) -> usize {
        let mut count = 0usize;
        // Build char_class → id mapping from skilldesc data
        // D2R: *Id=number, skill=name, charclass is same
        for row in rows {
            let skill_id: u16 = match row.get("*Id").or_else(|| row.get("Id")).and_then(|v| v.parse().ok()) {
                Some(id) => id,
                None => continue,
            };
            let name_en = row.get("skillname")
                .or_else(|| row.get("*skillname")).or_else(|| row.get("skill"))
                .map(|s| s.to_string()).unwrap_or_default();
            let char_class = row.get("charclass").or_else(|| row.get("*charclass"))
                .map(|s| s.to_string()).unwrap_or_default();
            let skill_tab = row.get("skiltab").or_else(|| row.get("SkillTab"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let is_aura = row.get("passive").or_else(|| row.get("*passive"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) == 1
                || row.get("server")
                    .map(|v| v.contains("aura"))
                    .unwrap_or(false);

            if name_en.is_empty() { continue; }
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO skill_def (profile_id, skill_id, name_en, char_class, skill_tab, is_aura)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![self.profile_id, skill_id, name_en, char_class, skill_tab, is_aura as i32],
            ) {
                log::warn!("[importer] skill_def insert error for {}: {}", skill_id, e);
                continue;
            }
            count += 1;
        }

        // Also import skill_tab_def from skills.txt charclass tabs
        let mut tab_map: std::collections::HashMap<(String, u8), String> = std::collections::HashMap::new();
        for row in rows {
            let char_class = row.get("charclass").or_else(|| row.get("*charclass"))
                .map(|s| s.to_lowercase()).unwrap_or_default();
            if char_class.is_empty() { continue; }
            if let Some(tab_raw) = row.get("skiltab").or_else(|| row.get("SkillTab"))
                && let Ok(tab) = tab_raw.parse::<u8>() {
                    let name_en = row.get("skillname")
                        .or_else(|| row.get("*skillname"))
                        .map(|s| s.to_string()).unwrap_or_default();
                    let key = (char_class, tab);
                    tab_map.entry(key).or_insert(name_en);
                }
        }
        for ((cl, tab), name) in &tab_map {
            let tab = *tab;
            let tab_id = match cl.as_str() {
                "ama" => tab,
                "nec" => tab + 3,
                "pal" => tab + 6,
                "bar" => tab + 9,
                "dru" => tab + 12,
                "ass" => tab + 15,
                "sor" => tab + 18,
                _ => continue,
            };
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO skill_tab_def (profile_id, tab_id, name_en, char_class)
                 VALUES (?1, ?2, ?3, ?4)",
                params![self.profile_id, tab_id, name, cl],
            ) {
                log::warn!("[importer] skill_tab_def insert error: {}", e);
            }
        }

        count
    }

    // ── Phase 2.2: MagicPrefix / MagicSuffix affix import ──

    /// Import item_affix_def from MagicPrefix.txt + MagicSuffix.txt.
    /// No hardcoded fallback — silently skipped when TXT files unavailable.
    pub fn import_item_affix(&self, excel_path: &str) -> ImportResult {
        let start = std::time::Instant::now();
        let mut count: usize = 0;
        let mut source: &str = "none";

        // MagicPrefix.txt: affix_id 0..727
        if let Some(path) = resolve_txt_path(excel_path, "MagicPrefix.txt")
            && let Some(rows) = read_txt_file(&path) {
                source = "MagicPrefix.txt";
                count += self._import_affix_from_txt(&rows, 0, "prefix");
            }

        // MagicSuffix.txt: affix_id 728..1456
        if let Some(path) = resolve_txt_path(excel_path, "MagicSuffix.txt")
            && let Some(rows) = read_txt_file(&path) {
                source = "MagicSuffix.txt";
                count += self._import_affix_from_txt(&rows, 728, "suffix");
            }

        let elapsed = start.elapsed();
        self._log_import("item_affix_def", count as i64, source, elapsed);
        ImportResult { table: "item_affix_def".to_string(), rows: count, source: source.to_string(), elapsed }
    }

    fn _import_affix_from_txt(&self, rows: &[HashMap<String, String>], base_id: u16, affix_type: &str) -> usize {
        let mut count = 0usize;
        for (offset, row) in rows.iter().enumerate() {
            let affix_id = base_id + offset as u16;
            let name_en = row.get("Name").or_else(|| row.get("name"))
                .map(|s| s.to_string()).unwrap_or_default();
            if name_en.is_empty() {
                continue;
            }
            let group_id = row.get("group").or_else(|| row.get("*group"))
                .and_then(|v| v.parse::<u16>().ok()).unwrap_or(0);
            let level = row.get("level").and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let level_req = row.get("levelreq").or_else(|| row.get("*levelreq"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let exclude_level = row.get("excludelevel").or_else(|| row.get("*excludelevel"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);
            let exclude_counter = row.get("excludecounter").or_else(|| row.get("*excludecounter"))
                .and_then(|v| v.parse::<u8>().ok()).unwrap_or(0);

            // mod1code + mod1param + mod1min + mod1max (first modifier)
            let mod1code = row.get("mod1code").or_else(|| row.get("*mod1code"))
                .map(|s| s.to_string()).unwrap_or_default();
            let mod1param = row.get("mod1param").or_else(|| row.get("*mod1param"))
                .and_then(|v| v.parse::<u16>().ok()).unwrap_or(0);
            let mod1min = row.get("mod1min").or_else(|| row.get("*mod1min"))
                .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
            let mod1max = row.get("mod1max").or_else(|| row.get("*mod1max"))
                .and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);

            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO item_affix_def
                    (profile_id, affix_id, name_en, affix_type, group_id,
                     level, level_req, exclude_level, exclude_counter,
                     mod1code, mod1param, mod1min, mod1max)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    self.profile_id, affix_id, name_en, affix_type, group_id,
                    level, level_req, exclude_level, exclude_counter,
                    mod1code, mod1param, mod1min, mod1max,
                ],
            ) {
                log::warn!("[importer] item_affix_def insert error for {}: {}", affix_id, e);
                continue;
            }
            count += 1;
        }
        count
    }

    // ── Phase 2.8: Item Type Localizations ──

    /// Import item type display names into `localized_string` with namespace `item_types`.
    /// Sources from D2R game string files.
    pub fn import_item_types(&self) -> ImportResult {
        let start = std::time::Instant::now();
        let mut count = 0usize;

        // Check if already imported
        let existing: i64 = self.conn.query_row(
            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1 AND namespace = 'item_types'",
            params![self.profile_id],
            |row| row.get(0),
        ).unwrap_or(0);
        if existing > 0 {
            let elapsed = start.elapsed();
            self._log_import("item_types", existing, "existing", elapsed);
            return ImportResult { table: "item_types".to_string(), rows: existing as usize, source: "existing".to_string(), elapsed };
        }

        // (kind, zhCN, zhTW) — extracted from D2R game string files
        let entries = [
            ("armor", "护甲", "護甲"),
            ("weapon", "武器", "武器"),
            ("shield", "盾牌", "盾牌"),
            ("rune", "符文", "符文"),
            ("gem", "宝石", "寶石"),
            ("jewel", "珠宝", "珠寶"),
            ("charm", "护身符", "護身符"),
            ("ring", "戒指", "戒指"),
            ("amulet", "项链", "項鍊"),
            ("potion", "药水", "藥水"),
            ("key", "钥匙", "鑰匙"),
            ("essence", "精华", "精華"),
            ("token", "徽章", "徽章"),
            ("shard", "碎片", "碎片"),
            ("quest", "任务", "任務"),
            ("book", "书籍", "書籍"),
            ("scroll", "卷轴", "卷軸"),
            ("misc", "杂项", "雜項"),
        ];

        for &(kind, zh_cn, zh_tw) in &entries {
            for (lang, val) in [("zhCN", zh_cn), ("zhTW", zh_tw)] {
                if let Err(e) = self.conn.execute(
                    "INSERT OR REPLACE INTO localized_string
                        (profile_id, namespace, string_key, language, text_value, source_path)
                     VALUES (?1, 'item_types', ?2, ?3, ?4, 'embedded')",
                    params![self.profile_id, kind, lang, val],
                ) {
                    log::warn!("[importer] item_types insert error for {}: {}", kind, e);
                    continue;
                }
                count += 1;
            }
        }

        let elapsed = start.elapsed();
        self._log_import("item_types", count as i64, "embedded", elapsed);
        ImportResult { table: "item_types".to_string(), rows: count, source: "embedded".to_string(), elapsed }
    }

    // ── Import logging ──

    fn _log_import(&self, table_name: &str, rows: i64, source: &str, elapsed: std::time::Duration) {
        // Delete old entry for this table (if reimported), then insert new one
        let _ = self.conn.execute(
            "DELETE FROM resource_import_log WHERE profile_id = ?1 AND table_name = ?2",
            params![self.profile_id, table_name],
        );
        if let Err(e) = self.conn.execute(
            "INSERT INTO resource_import_log
                (profile_id, table_name, rows_count, source, completed_at, status)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, 'completed')",
            params![self.profile_id, table_name, rows, source],
        ) {
            log::warn!("[importer] failed to log import to {}: {}", table_name, e);
        }
        log::info!(
            "[importer] {}: {} rows from {} in {:?}",
            table_name, rows, source, elapsed,
        );
    }
}

/// Result of a single table import.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub table: String,
    pub rows: usize,
    pub source: String,
    pub elapsed: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::db::Database;
    use crate::resource::queries;
    use rusqlite::Connection;

    fn setup_import_conn() -> (Connection, i64) {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        let db = Database::init_from_connection(conn).expect("init database schema");
        let conn = db.into_connection();
        conn.execute(
            "INSERT INTO resource_profile (profile_key, source_kind, excel_path)
             VALUES ('test:mod:3.2', 'mod', '')",
            [],
        )
        .expect("insert resource profile");
        let profile_id = conn
            .query_row("SELECT id FROM resource_profile WHERE profile_key = 'test:mod:3.2'", [], |row| {
                row.get(0)
            })
            .expect("query profile_id");
        (conn, profile_id)
    }

    #[test]
    fn test_import_item_base_from_constants_marks_grimoire_as_offhand() {
        let (conn, profile_id) = setup_import_conn();
        let importer = ResourceImporter::new(&conn, profile_id);

        let result = importer.import_item_base("Z:/path/that/does/not/exist");
        assert_eq!(result.source, "const");

        let wae = queries::get_item_base(&conn, profile_id, "wae").expect("wae should exist in fallback constants");
        assert_eq!(wae.item_category, "offhand");
        assert!(!wae.is_armor, "grimoire should no longer be exposed as armor in item_base");
        assert!(!wae.is_weapon, "grimoire should no longer be exposed as weapon in item_base");
        assert!(wae.is_shield, "grimoire should be queryable as shield/offhand");
    }

    #[test]
    fn test_import_item_base_from_txt_overrides_armor_category_for_grimoire() {
        let (conn, profile_id) = setup_import_conn();
        let importer = ResourceImporter::new(&conn, profile_id);

        let temp_root = std::env::temp_dir().join(format!("d2r_importer_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_root).expect("create temp dir");
        let armor_txt = temp_root.join("armor.txt");
        std::fs::write(
            &armor_txt,
            "name\tcode\ttype\tinvwidth\tinvheight\tstackable\thasinv\tlevel\tlevelreq\tgemapply\n\
Blasphemous Compendium\twae\twand\t2\t2\t0\t1\t90\t80\t\n",
        )
        .expect("write armor.txt");

        let result = importer.import_item_base(temp_root.to_string_lossy().as_ref());
        assert_eq!(result.source, "misc.txt,armor.txt,weapons.txt");

        let wae = queries::get_item_base(&conn, profile_id, "wae").expect("wae should be imported from txt");
        assert_eq!(wae.item_type, "wand");
        assert_eq!(wae.item_category, "offhand");
        assert_eq!((wae.inv_width, wae.inv_height), (2, 2));
        assert!(!wae.is_armor);
        assert!(!wae.is_weapon);
        assert!(wae.is_shield);

        std::fs::remove_dir_all(&temp_root).ok();
    }
}
