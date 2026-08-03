//! 圣杯追踪命令。
//!
//! 记录已发现/未发现的暗金和套装物品。

use crate::AppState;
use crate::commands::config::get_active_profile_id;
use crate::resource::queries;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct GrailProgress {
    pub total: usize,
    pub found: usize,
    pub pct: f64,
    pub items: Vec<GrailItemView>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrailItemView {
    pub item_key: String,
    pub item_type: String,     // "unique" or "set"
    pub name_en: String,
    pub item_code: String,
    pub level: u8,
    pub found: bool,
    pub found_at: Option<String>,
}

/// 初始化圣杯数据：如果当前 profile 还没有圣杯记录，从 unique/set 定义表导入
fn ensure_grail_entries(conn: &rusqlite::Connection, profile_id: i64) -> Result<(), String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM grail_tracking WHERE profile_id = ?1",
            params![profile_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if existing > 0 {
        return Ok(()); // already initialized
    }

    // Import unique items
    for u in queries::get_all_unique_defs(conn, profile_id) {
        let key = format!("unique:{}", u.unique_id);
        conn.execute(
            "INSERT OR IGNORE INTO grail_tracking (profile_id, item_key, item_type, item_code, name_en, found)
             VALUES (?1, ?2, 'unique', ?3, ?4, 0)",
            params![profile_id, key, u.base_code, u.name_en],
        ).map_err(|e| e.to_string())?;
    }

    // Import set items
    for s in queries::get_all_set_item_defs(conn, profile_id) {
        let key = format!("set:{}", s.item_id);
        conn.execute(
            "INSERT OR IGNORE INTO grail_tracking (profile_id, item_key, item_type, item_code, name_en, found)
             VALUES (?1, ?2, 'set', ?3, ?4, 0)",
            params![profile_id, key, s.base_code, s.name_en],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 获取圣杯进度
#[tauri::command]
pub fn get_grail(state: State<AppState>) -> Result<GrailProgress, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = get_active_profile_id(&db).unwrap_or(0);
    if profile_id == 0 {
        return Err("No active profile".into());
    }

    ensure_grail_entries(conn, profile_id)?;

    let entries = queries::get_grail_entries(conn, profile_id);
    let all_unique = queries::get_all_unique_defs(conn, profile_id);
    let all_set = queries::get_all_set_item_defs(conn, profile_id);

    // Build level map for items
    let mut level_map: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
    for u in &all_unique {
        level_map.insert(format!("unique:{}", u.unique_id), u.level);
    }
    for s in &all_set {
        level_map.insert(format!("set:{}", s.item_id), s.level);
    }

    let total = entries.len();
    let found = entries.iter().filter(|e| e.found).count();

    let items: Vec<GrailItemView> = entries
        .into_iter()
        .map(|e| GrailItemView {
            item_key: e.item_key.clone(),
            item_type: e.item_type.clone(),
            name_en: e.name_en,
            item_code: e.item_code,
            level: level_map.get(&e.item_key).copied().unwrap_or(0),
            found: e.found,
            found_at: e.found_at,
        })
        .collect();

    let pct = if total > 0 { found as f64 / total as f64 * 100.0 } else { 0.0 };

    Ok(GrailProgress { total, found, pct, items })
}

/// 标记/取消标记圣杯物品
#[tauri::command]
pub fn toggle_grail(
    state: State<AppState>,
    item_key: String,
    found: bool,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = get_active_profile_id(&db).unwrap_or(0);
    if profile_id == 0 {
        return Err("No active profile".into());
    }

    if found {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE grail_tracking SET found = 1, found_at = ?1 WHERE profile_id = ?2 AND item_key = ?3",
            params![now, profile_id, item_key],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE grail_tracking SET found = 0, found_at = NULL WHERE profile_id = ?1 AND item_key = ?2",
            params![profile_id, item_key],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}
