//! 符文之语计算器命令。
//!
//! 输入已拥有的符文集合，返回所有可制作的符文之语。

use crate::AppState;
use crate::commands::config::get_active_profile_id;
use crate::resource::queries;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::State;

/// 符文之语计算结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RunewordMatch {
    pub name_en: String,
    pub name_zh: Option<String>,
    pub runes: Vec<String>,
    pub allowed_bases: Vec<String>,
    pub sockets: u8,
}

/// 查询可制作的符文之语
#[tauri::command]
pub fn find_runewords(
    state: State<AppState>,
    owned_runes: Vec<String>,
) -> Result<Vec<RunewordMatch>, String> {
    let owned: HashSet<String> = owned_runes.into_iter().collect();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = get_active_profile_id(&db).unwrap_or(0);
    if profile_id == 0 {
        return Err("No active profile — run resource import first".into());
    }

    let matches = queries::find_runewords_by_runes(conn, profile_id, &owned);
    let result: Vec<RunewordMatch> = matches
        .into_iter()
        .map(|rw| {
            let runes: Vec<String> = rw.rune_codes.split(',')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect();
            let bases: Vec<String> = rw.allowed_base_types.split(',')
                .map(|b| b.trim().to_string())
                .filter(|b| !b.is_empty())
                .collect();
            // Look up Chinese name: zhCN first, then zhTW (D2R has no zhCN for many entries)
            let rw_key_lower = rw.runeword_key.to_lowercase();
            let name_zh = conn
                .prepare_cached(
                    "SELECT text_value FROM localized_string
                     WHERE profile_id = ?1 AND namespace = 'item_runes' AND string_key = ?2 AND language = ?3
                     LIMIT 1",
                )
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_row(params![profile_id, rw_key_lower, "zhCN"], |row| row.get(0)).ok()
                })
                .or_else(|| {
                    conn.prepare_cached(
                        "SELECT text_value FROM localized_string
                         WHERE profile_id = ?1 AND namespace = 'item_runes' AND string_key = ?2 AND language = ?3
                         LIMIT 1",
                    )
                    .ok()
                    .and_then(|mut stmt| {
                        stmt.query_row(params![profile_id, rw_key_lower, "zhTW"], |row| row.get(0)).ok()
                    })
                });
            RunewordMatch {
                name_en: rw.name_en,
                name_zh,
                runes,
                allowed_bases: bases,
                sockets: rw.sockets,
            }
        })
        .collect();

    Ok(result)
}


/// 已拥有的符文 + 带孔物品的 item_type（用于前端高亮可制作符文之语）
#[derive(Debug, Serialize, Deserialize)]
pub struct RunewordContext {
    pub owned_runes: Vec<String>,
    /// 带孔物品的 item_type（小写），例如 "swor" "shld"
    pub socketed_base_types: Vec<String>,
}

#[tauri::command]
pub fn get_runeword_context(state: State<AppState>) -> Result<RunewordContext, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = get_active_profile_id(&db).unwrap_or(0);

    // 1. Owned runes from warehouse
    let mut owned_runes: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT item_code FROM warehouse_items WHERE item_kind = 'rune' AND item_code LIKE 'r%'"
    )
        && let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                if !owned_runes.contains(&row) {
                    owned_runes.push(row);
                }
            }
        }

    // 2. Socketed base items: query item_json for socketed=true items
    //    and map their item_code to item_type via item_base table
    let mut socketed_types: Vec<String> = Vec::new();
    if profile_id > 0
        && let Ok(mut stmt) = conn.prepare(
            "SELECT DISTINCT ib.item_type
             FROM warehouse_items w
             JOIN item_base ib ON ib.code = w.item_code AND ib.profile_id = ?1
             WHERE w.item_kind NOT IN ('rune','gem','jewel','potion','scroll','misc')
               AND json_extract(w.item_json, '$.socketed') = 'true'
               AND ib.item_type IS NOT NULL AND ib.item_type != ''"
        )
            && let Ok(rows) = stmt.query_map(rusqlite::params![profile_id], |row| row.get::<_, String>(0)) {
                for row in rows.flatten() {
                    if !socketed_types.contains(&row) {
                        socketed_types.push(row);
                    }
                }
            }

    // Sort for determinism
    owned_runes.sort();
    socketed_types.sort();

    Ok(RunewordContext {
        owned_runes,
        socketed_base_types: socketed_types,
    })
}
