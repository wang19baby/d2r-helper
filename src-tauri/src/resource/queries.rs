//! Query layer for SQLite definition tables.
//!
//! Provides typed access to item_base, unique_item_def, set_def, set_item_def tables.
//! All queries require a profile_id to maintain multi-profile isolation.

use rusqlite::{Connection, params};

/// Basic item type definition from item_base table.
#[derive(Debug, Clone)]
pub struct ItemBaseRow {
    pub code: String,
    pub name_en: String,
    pub item_type: String,
    pub item_category: String,
    pub inv_width: u8,
    pub inv_height: u8,
    pub stackable: bool,
    pub has_inventory: bool,
    pub level: u8,
    pub level_req: u8,
    pub is_armor: bool,
    pub is_weapon: bool,
    pub is_shield: bool,
    pub is_mod_item: bool,
}

/// Unique item definition from unique_item_def table.
#[derive(Debug, Clone)]
pub struct UniqueItemDefRow {
    pub unique_id: u16,
    pub name_en: String,
    pub base_code: String,
    pub level: u8,
    pub level_req: u8,
    pub is_mod_item: bool,
}

/// Set definition from set_def table.
#[derive(Debug, Clone)]
pub struct SetDefRow {
    pub set_id: u16,
    pub name_en: String,
}

/// Set item definition from set_item_def table.
#[derive(Debug, Clone)]
pub struct SetItemDefRow {
    pub item_id: u16,
    pub set_id: u16,
    pub name_en: String,
    pub base_code: String,
    pub level: u8,
    pub level_req: u8,
}

/// Set partial/full bonus row from set_bonus_def.
#[derive(Debug, Clone)]
pub struct SetBonusDefRow {
    pub piece_count: u8,
    pub stat_id: u16,
    pub param: i32,
    pub min_value: i32,
    pub max_value: i32,
}

/// Import log entry from resource_import_log table.
#[derive(Debug, Clone)]
pub struct ImportLogRow {
    pub table_name: String,
    pub rows_count: i64,
    pub source: String,
    pub status: String,
}

/// Query a single item base row by code.
pub fn get_item_base(conn: &Connection, profile_id: i64, code: &str) -> Option<ItemBaseRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT code, name_en, item_type, item_category,
                    inv_width, inv_height, stackable, has_inventory,
                    level, level_req, is_armor, is_weapon, is_shield, is_mod_item
             FROM item_base
             WHERE profile_id = ?1 AND code = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, code], |row| {
        Ok(ItemBaseRow {
            code: row.get(0)?,
            name_en: row.get(1)?,
            item_type: row.get(2)?,
            item_category: row.get(3)?,
            inv_width: row.get(4)?,
            inv_height: row.get(5)?,
            stackable: row.get::<_, i32>(6)? != 0,
            has_inventory: row.get::<_, i32>(7)? != 0,
            level: row.get(8)?,
            level_req: row.get(9)?,
            is_armor: row.get::<_, i32>(10)? != 0,
            is_weapon: row.get::<_, i32>(11)? != 0,
            is_shield: row.get::<_, i32>(12)? != 0,
            is_mod_item: row.get::<_, i32>(13)? != 0,
        })
    })
    .ok()
}

/// Query a unique item definition by unique_id.
pub fn get_unique_def(conn: &Connection, profile_id: i64, unique_id: u16) -> Option<UniqueItemDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT unique_id, name_en, base_code, level, level_req, is_mod_item
             FROM unique_item_def
             WHERE profile_id = ?1 AND unique_id = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, unique_id], |row| {
        Ok(UniqueItemDefRow {
            unique_id: row.get(0)?,
            name_en: row.get(1)?,
            base_code: row.get(2)?,
            level: row.get(3)?,
            level_req: row.get(4)?,
            is_mod_item: row.get::<_, i32>(5)? != 0,
        })
    })
    .ok()
}

/// Query a set definition by set_id.
pub fn get_set_def(conn: &Connection, profile_id: i64, set_id: u16) -> Option<SetDefRow> {
    let mut stmt = conn
        .prepare_cached("SELECT set_id, name_en FROM set_def WHERE profile_id = ?1 AND set_id = ?2")
        .ok()?;
    stmt.query_row(params![profile_id, set_id], |row| {
        Ok(SetDefRow {
            set_id: row.get(0)?,
            name_en: row.get(1)?,
        })
    })
    .ok()
}

/// Query all set items belonging to a set (by set group id).
pub fn get_set_items_by_set(conn: &Connection, profile_id: i64, set_id: u16) -> Vec<SetItemDefRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT item_id, set_id, name_en, base_code, level, level_req
         FROM set_item_def
         WHERE profile_id = ?1 AND set_id = ?2
         ORDER BY item_id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id, set_id], |row| {
        Ok(SetItemDefRow {
            item_id: row.get(0)?,
            set_id: row.get(1)?,
            name_en: row.get(2)?,
            base_code: row.get(3)?,
            level: row.get(4)?,
            level_req: row.get(5)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// Query a set item definition by its item_id.
/// 存档里套装物品的 set_id 字段 = setitems.txt 的 *ID (item_id), 不是 set_def 组 id。
pub fn get_set_item_by_item_id(conn: &Connection, profile_id: i64, item_id: u16) -> Option<SetItemDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT item_id, set_id, name_en, base_code, level, level_req
             FROM set_item_def
             WHERE profile_id = ?1 AND item_id = ?2
             LIMIT 1",
        )
        .ok()?;
    stmt.query_row(params![profile_id, item_id], |row| {
        Ok(SetItemDefRow {
            item_id: row.get(0)?,
            set_id: row.get(1)?,
            name_en: row.get(2)?,
            base_code: row.get(3)?,
            level: row.get(4)?,
            level_req: row.get(5)?,
        })
    })
    .ok()
}

/// Query set bonuses by set group id (2-5 partial + 6 full set).
pub fn get_set_bonuses_by_set(conn: &Connection, profile_id: i64, set_id: u16) -> Vec<SetBonusDefRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT piece_count, stat_id, param, min_value, max_value
         FROM set_bonus_def
         WHERE profile_id = ?1 AND set_id = ?2
         ORDER BY piece_count, stat_id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id, set_id], |row| {
        Ok(SetBonusDefRow {
            piece_count: row.get(0)?,
            stat_id: row.get(1)?,
            param: row.get(2)?,
            min_value: row.get(3)?,
            max_value: row.get(4)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// Get all item codes registered for a profile.
pub fn get_all_item_codes(conn: &Connection, profile_id: i64) -> Vec<String> {
    let mut stmt = match conn.prepare_cached(
        "SELECT code FROM item_base WHERE profile_id = ?1 ORDER BY code",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| row.get(0))
        .ok()
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
}

/// Get import log entries for a profile.
pub fn get_import_log(conn: &Connection, profile_id: i64) -> Vec<ImportLogRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT table_name, rows_count, source, status
         FROM resource_import_log
         WHERE profile_id = ?1
         ORDER BY table_name",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| {
        Ok(ImportLogRow {
            table_name: row.get(0)?,
            rows_count: row.get(1)?,
            source: row.get(2)?,
            status: row.get(3)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

// ── Phase 2.2: Runeword queries ──

/// Runeword definition from runeword_def table.
#[derive(Debug, Clone)]
pub struct RunewordDefRow {
    pub runeword_key: String,
    pub name_en: String,
    pub rune_codes: String,
    pub allowed_base_types: String,
    pub sockets: u8,
}

/// Get all runeword definitions for a profile.
pub fn get_all_runewords(conn: &Connection, profile_id: i64) -> Vec<RunewordDefRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT runeword_key, name_en, rune_codes, allowed_base_types, sockets
         FROM runeword_def WHERE profile_id = ?1 ORDER BY sockets, runeword_key",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| {
        Ok(RunewordDefRow {
            runeword_key: row.get(0)?,
            name_en: row.get(1)?,
            rune_codes: row.get(2)?,
            allowed_base_types: row.get(3)?,
            sockets: row.get(4)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// Find runewords that can be made with a given set of owned rune codes.
///
/// `owned_runes`: set of rune codes the user has (e.g. {"r01", "r02", "r03"}).
/// A runeword matches if ALL its required runes are in the owned set.
/// Returns runewords sorted by socket count ascending.
pub fn find_runewords_by_runes(
    conn: &Connection,
    profile_id: i64,
    owned_runes: &std::collections::HashSet<String>,
) -> Vec<RunewordDefRow> {
    let all = get_all_runewords(conn, profile_id);
    all.into_iter()
        .filter(|rw| {
            rw.rune_codes.split(',')
                .map(|r| r.trim().to_string())
                .all(|rune| owned_runes.contains(&rune))
        })
        .collect()
}

/// Look up a runeword by key (e.g. "Runeword11").
pub fn get_runeword_by_key(conn: &Connection, profile_id: i64, key: &str) -> Option<RunewordDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT runeword_key, name_en, rune_codes, allowed_base_types, sockets
             FROM runeword_def WHERE profile_id = ?1 AND runeword_key = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, key], |row| {
        Ok(RunewordDefRow {
            runeword_key: row.get(0)?,
            name_en: row.get(1)?,
            rune_codes: row.get(2)?,
            allowed_base_types: row.get(3)?,
            sockets: row.get(4)?,
        })
    })
    .ok()
}

// ── Phase 2.2: Stat queries ──

/// Stat definition from stat_def table.
#[derive(Debug, Clone)]
pub struct StatDefRow {
    pub stat_id: u16,
    pub name_en: String,
    pub save_bits: u8,
    pub save_param_bits: u8,
    pub save_add: i32,
    pub signed: bool,
    pub encoding: u8,
}

/// Get stat definition by stat ID.
pub fn get_stat_def(conn: &Connection, profile_id: i64, stat_id: u16) -> Option<StatDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT stat_id, name_en, save_bits, save_param_bits, save_add, signed, encoding
             FROM stat_def WHERE profile_id = ?1 AND stat_id = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, stat_id], |row| {
        Ok(StatDefRow {
            stat_id: row.get(0)?,
            name_en: row.get(1)?,
            save_bits: row.get(2)?,
            save_param_bits: row.get(3)?,
            save_add: row.get(4)?,
            signed: row.get::<_, i32>(5)? != 0,
            encoding: row.get(6)?,
        })
    })
    .ok()
}

// ── Phase 2.2: Skill queries ──

/// Skill definition from skill_def table.
#[derive(Debug, Clone)]
pub struct SkillDefRow {
    pub skill_id: u16,
    pub name_en: String,
    pub char_class: String,
    pub skill_tab: u8,
    pub is_aura: bool,
}

/// Get skill definition by skill ID.
pub fn get_skill_def(conn: &Connection, profile_id: i64, skill_id: u16) -> Option<SkillDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT skill_id, name_en, char_class, skill_tab, is_aura
             FROM skill_def WHERE profile_id = ?1 AND skill_id = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, skill_id], |row| {
        Ok(SkillDefRow {
            skill_id: row.get(0)?,
            name_en: row.get(1)?,
            char_class: row.get(2)?,
            skill_tab: row.get(3)?,
            is_aura: row.get::<_, i32>(4)? != 0,
        })
    })
    .ok()
}

/// Skill tab definition from skill_tab_def table.
#[derive(Debug, Clone)]
pub struct SkillTabDefRow {
    pub tab_id: u8,
    pub name_en: String,
    pub char_class: String,
}

/// Get skill tab name by tab_id.
pub fn get_skill_tab_def(conn: &Connection, profile_id: i64, tab_id: u8) -> Option<SkillTabDefRow> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT tab_id, name_en, char_class FROM skill_tab_def WHERE profile_id = ?1 AND tab_id = ?2",
        )
        .ok()?;
    stmt.query_row(params![profile_id, tab_id], |row| {
        Ok(SkillTabDefRow {
            tab_id: row.get(0)?,
            name_en: row.get(1)?,
            char_class: row.get(2)?,
        })
    })
    .ok()
}

// ── Grail queries ──

/// 获取所有 unique 物品定义（用于圣杯列表）
pub fn get_all_unique_defs(conn: &Connection, profile_id: i64) -> Vec<UniqueItemDefRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT unique_id, name_en, base_code, level, level_req, is_mod_item
         FROM unique_item_def WHERE profile_id = ?1 ORDER BY level, unique_id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| {
        Ok(UniqueItemDefRow {
            unique_id: row.get(0)?,
            name_en: row.get(1)?,
            base_code: row.get(2)?,
            level: row.get(3)?,
            level_req: row.get(4)?,
            is_mod_item: row.get::<_, i32>(5)? != 0,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// 获取所有 set 物品定义（用于圣杯列表）
pub fn get_all_set_item_defs(conn: &Connection, profile_id: i64) -> Vec<SetItemDefRow> {
    let mut stmt = match conn.prepare_cached(
        "SELECT item_id, set_id, name_en, base_code, level, level_req
         FROM set_item_def WHERE profile_id = ?1 ORDER BY level, set_id, item_id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| {
        Ok(SetItemDefRow {
            item_id: row.get(0)?,
            set_id: row.get(1)?,
            name_en: row.get(2)?,
            base_code: row.get(3)?,
            level: row.get(4)?,
            level_req: row.get(5)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// 获取圣杯进度
pub fn get_grail_entries(conn: &Connection, profile_id: i64) -> Vec<crate::database::GrailEntry> {
    let mut stmt = match conn.prepare_cached(
        "SELECT item_key, item_type, item_code, name_en, found, found_at
         FROM grail_tracking WHERE profile_id = ?1 ORDER BY item_key",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![profile_id], |row| {
        Ok(crate::database::GrailEntry {
            profile_id,
            item_key: row.get(0)?,
            item_type: row.get(1)?,
            item_code: row.get(2)?,
            name_en: row.get(3)?,
            found: row.get::<_, i32>(4)? != 0,
            found_at: row.get(5)?,
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

// ── Phase 2.2: Affix (MagicPrefix / MagicSuffix) queries ──

/// Get an affix display name by its affix_id.
///
/// `affix_id` 0-727 = MagicPrefix, 728-1456 = MagicSuffix.
/// Returns the English name from item_affix_def table.
pub fn get_affix_name(conn: &Connection, profile_id: i64, affix_id: u16) -> Option<String> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT name_en FROM item_affix_def
             WHERE profile_id = ?1 AND affix_id = ?2
             LIMIT 1",
        )
        .ok()?;
    stmt.query_row(params![profile_id, affix_id], |row| row.get::<_, String>(0))
        .ok()
}

/// Get both prefix and suffix names from affix IDs.
///
/// prefix_id 0-727 for MagicPrefix, suffix_id 728-1456 for MagicSuffix.
/// Returns (Option<prefix_name>, Option<suffix_name>).
pub fn get_affix_name_pair(
    conn: &Connection,
    profile_id: i64,
    prefix_id: u16,
    suffix_id: u16,
) -> (Option<String>, Option<String>) {
    let prefix = get_affix_name(conn, profile_id, prefix_id);
    let suffix = get_affix_name(conn, profile_id, suffix_id);
    (prefix, suffix)
}

/// 获取单个词缀的等级需求。
pub fn get_affix_level_req(conn: &Connection, profile_id: i64, affix_id: u16) -> Option<u8> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT level_req FROM item_affix_def
             WHERE profile_id = ?1 AND affix_id = ?2
             LIMIT 1",
        )
        .ok()?;
    stmt.query_row(params![profile_id, affix_id], |row| row.get::<_, u8>(0))
        .ok()
}

/// 获取魔法物品的最高词缀等级需求（取 prefix/suffix 中较大的）。
/// suffix_id 是 D2S 文件中的 0-based 索引。
pub fn get_magic_item_req_level(conn: &Connection, profile_id: i64, prefix_id: Option<u16>, suffix_id: Option<u16>) -> u8 {
    let p_lvl = prefix_id.and_then(|id| get_prefix_level_req(conn, profile_id, id)).unwrap_or(0);
    let s_lvl = suffix_id.and_then(|id| get_suffix_level_req(conn, profile_id, id)).unwrap_or(0);
    std::cmp::max(p_lvl, s_lvl)
}

/// 查询前缀（MagicPrefix）等级需求，id 为 0-based index。
fn get_prefix_level_req(conn: &Connection, profile_id: i64, id: u16) -> Option<u8> {
    get_affix_level_req(conn, profile_id, id)
}

/// 查询后缀（MagicSuffix）等级需求，id 为 0-based index。
/// 数据库内部用 728+id 存储。
fn get_suffix_level_req(conn: &Connection, profile_id: i64, id: u16) -> Option<u8> {
    get_affix_level_req(conn, profile_id, 728 + id)
}
