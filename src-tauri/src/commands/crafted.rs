//! 手工艺品上下文命令 — 从仓库查询可用于 crafted 配方的物品。

use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 仓库存货摘要，用于前端匹配手工艺配方
#[derive(Debug, Serialize, Deserialize)]
pub struct CraftedContext {
    /// 仓库中所有不同的物品代码（3-char code），如 "fhl", "ring", "amul"
    pub owned_codes: Vec<String>,
    /// 是否有魔法物品 (quality = 'magic')
    pub has_magic: bool,
    /// 是否有升级/合成材料（upg 标记对应的物品）
    pub has_upgrades: bool,
}

/// 查询仓库中可用于手工装备 (Crafted Items) 的物品摘要
#[tauri::command]
pub fn get_crafted_context(state: State<AppState>) -> Result<CraftedContext, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();

    // 1. 所有不同的 item_code
    let mut owned_codes: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT item_code FROM warehouse_items ORDER BY item_code"
    )
        && let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                owned_codes.push(row);
            }
        }

    // 2. 是否有魔法物品
    let has_magic = conn
        .query_row(
            "SELECT COUNT(*) FROM warehouse_items WHERE quality = 'magic'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    // 3. 是否有升级/合成材料（crafted 配方的 upg 标记）
    // Crushing Blow 类：特定 misc/quest 物品
    // 只要仓库里有非 rune/non-tradeable 的物品即可
    let has_upgrades = conn
        .query_row(
            "SELECT COUNT(*) FROM warehouse_items
             WHERE item_kind NOT IN ('rune', 'gem', 'jewel', 'potion', 'scroll')
               AND simple_item = 1
               AND quality = 'normal'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    Ok(CraftedContext {
        owned_codes,
        has_magic,
        has_upgrades,
    })
}
