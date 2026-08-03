//! 交易历史命令
//!
//! 读取 `transactions` 表,返回前端展示用结构。前端 History 页面消费。

use crate::database::Transaction;
use crate::AppState;
use serde::Serialize;
use tauri::State;

/// 前端友好的交易历史响应
#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub tx_type: String,
    pub item_id: Option<String>,
    pub token_amount: i64,
    pub description: String,
    pub date: Option<String>,
}

impl From<Transaction> for HistoryEntry {
    fn from(t: Transaction) -> Self {
        HistoryEntry {
            id: t.id.unwrap_or(0),
            tx_type: t.tx_type,
            item_id: t.item_id,
            token_amount: t.token_amount,
            description: t.description,
            date: t.date,
        }
    }
}

/// 获取交易历史(默认最新在前)。
///
/// - `limit`: 最多返回 N 条,默认 200, 上限 1000
/// - `tx_type`: 可选, 只返回指定类型(空字符串 = 不过滤)
#[tauri::command]
pub fn get_transactions(
    state: State<AppState>,
    limit: Option<i64>,
    tx_type: Option<String>,
) -> Result<Vec<HistoryEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let tx_type_filter: Option<&str> = tx_type
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    db.get_transactions(limit.unwrap_or(200), tx_type_filter)
        .map(|v| v.into_iter().map(HistoryEntry::from).collect())
        .map_err(|e| e.to_string())
}