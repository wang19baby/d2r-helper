use crate::AppState;
use tauri::State;

/// Get the current token balance
#[tauri::command]
pub fn get_balance(state: State<AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_token_balance().map_err(|e| e.to_string())
}
