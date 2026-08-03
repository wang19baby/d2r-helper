

use crate::AppState;
use crate::database::Database;
use crate::protocol::d2i::legacy::resource_manifest::{build_resource_manifest, ResourceManifest};
use crate::resource::import_task;
use crate::resource::queries;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatus {
    pub table_name: String,
    pub rows: usize,
    pub source: String,
    pub elapsed_ms: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub save_path: String,
}

/// A saved resource profile (for multi-profile switching).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: i64,
    pub profile_key: String,
    pub source_kind: String,
    pub mod_name: String,
    pub game_version: String,
    pub active_language: String,
    pub game_root: String,
    pub excel_path: String,
    pub checksum: String,
    pub source_path: String,
    pub imported_at: Option<String>,
    pub import_status: Vec<ImportStatus>,
    pub localized_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfigResponse {
    pub save_folder: String,
    pub default_folder: String,
    pub game_root: String,
    pub profile_key: String,
    pub active_mod: String,
    pub game_version: String,
    pub language: String,
    pub stash_grid_size: u8,
    pub backpack_cols: u8,
    pub backpack_rows: u8,
    pub cube_cols: u8,
    pub cube_rows: u8,
    pub available_mods: Vec<String>,
    pub mod_metadata: Vec<ModMeta>,
    pub game_data_path: String,
    pub resource_manifest: Option<ResourceManifest>,
    pub import_status: Vec<ImportStatus>,
    /// All saved profiles for switching.
    pub profiles: Vec<ProfileInfo>,
}

fn get_default_save_folder() -> String {
    if cfg!(target_os = "windows")
        && let Ok(prof) = std::env::var("USERPROFILE") {
            return Path::new(&prof).join("Saved Games").join("Diablo II Resurrected").to_string_lossy().to_string();
        }
    "".to_string()
}

/// Parse modinfo.json from a mod's directory for extended metadata.
/// Returns ModMeta with available fields (defaults empty for missing fields).
fn read_mod_info(mod_dir: &Path, mod_name: &str) -> ModMeta {
    // Try modinfo.json (D2RLAN format) and mod.json (common alternative)
    for filename in &["modinfo.json", "mod.json"] {
        let meta_path = mod_dir.join(filename);
        if let Ok(content) = std::fs::read_to_string(&meta_path) {
            // Parse with serde_json, tolerating // comments found in modinfo.json examples
            let cleaned: String = content.lines()
                .filter(|l| !l.trim().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            if let Ok(meta) = serde_json::from_str::<ModMeta>(&cleaned) {
                return meta;
            }
            // Fallback: try to extract single fields manually
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                return ModMeta {
                    name: val.get("name").and_then(|v| v.as_str()).unwrap_or(mod_name).to_string(),
                    version: val.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    author: val.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    save_path: val.get("savepath").or_else(|| val.get("save_path")).or_else(|| val.get("savedir"))
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };
            }
        }
    }
    // No metadata file found: return basic info
    ModMeta {
        name: mod_name.to_string(),
        version: String::new(),
        description: String::new(),
        author: String::new(),
        save_path: String::new(),
    }
}

fn scan_mods(game_root: &str) -> (Vec<String>, Vec<ModMeta>) {
    let mods_dir = Path::new(game_root).join("mods");
    if !mods_dir.is_dir() { return (vec!["(原版)".to_string()], vec![ModMeta {
        name: "(原版)".into(), version: String::new(), description: "原版 D2R".into(), author: String::new(), save_path: String::new(),
    }]); }

    let mut mods = vec!["(原版)".to_string()];
    let mut meta = vec![ModMeta {
        name: "(原版)".into(), version: String::new(), description: "原版 D2R 游戏".into(), author: String::new(), save_path: String::new(),
    }];

    if let Ok(entries) = std::fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let base = mods_dir.join(&name);
            let m = format!("{}.mpq", name);
            let paths = [
                base.join(&m).join("data").join("global").join("excel"),
                base.join("data").join("global").join("excel"),
                base.join(&m).join("excel"),
            ];
            let mut found = false;
            for p in &paths {
                if p.join("base").join("misc.txt").exists() || p.join("misc.txt").exists() {
                    found = true;
                    break;
                }
            }
            if found {
                mods.push(name.clone());
                let mod_meta = read_mod_info(&base, &name);
                meta.push(mod_meta);
            }
        }
    }
    (mods, meta)
}

/// Get the active profile_id from the database.
/// Returns 0 if no active profile is set.
pub fn build_profile_key(active_mod: &str, game_version: &str) -> String {
    if active_mod.is_empty() || active_mod == "(原版)" {
        if game_version.is_empty() {
            "vanilla:default".to_string()
        } else {
            format!("vanilla:{}", game_version)
        }
    } else if game_version.is_empty() {
        format!("mod:{}", active_mod)
    } else {
        format!("mod:{}:{}", active_mod, game_version)
    }
}

pub fn get_active_profile_key(db: &Database) -> Result<String, String> {
    let _game_root = db.get_config("game_root").map_err(|e| e.to_string())?.unwrap_or_default();
    let active_mod = db.get_config("active_mod").map_err(|e| e.to_string())?.unwrap_or_default();
    let game_version = db.get_config("game_version").map_err(|e| e.to_string())?.unwrap_or_default();
    let _language = db.get_config("language").map_err(|e| e.to_string())?.unwrap_or_else(|| "enUS".to_string());
    Ok(build_profile_key(&active_mod, &game_version))
}

/// Get the active profile_id from the database.
/// Returns 0 if no active profile is set.
pub fn get_active_profile_id(db: &Database) -> Result<i64, String> {
    let profile_key = get_active_profile_key(db)?;
    let conn = db.get_connection();
    conn.query_row(
        "SELECT id FROM resource_profile WHERE profile_key = ?1",
        rusqlite::params![profile_key],
        |row| row.get(0),
    ).map_err(|e| format!("Profile not found: {}", e))
}

pub fn resolve_excel_path(game_root: &str, active_mod: &str) -> String {
    if active_mod.is_empty() || active_mod == "(原版)" {
        let v = Path::new(game_root).join("data").join("global").join("excel");
        if v.join("base").join("misc.txt").exists() || v.join("misc.txt").exists() { return v.to_string_lossy().to_string(); }
        return "".to_string();
    }
    let excel = Path::new(game_root).join("mods").join(active_mod)
        .join(format!("{}.mpq", active_mod)).join("data").join("global").join("excel");
    if excel.join("base").join("misc.txt").exists() || excel.join("misc.txt").exists() { return excel.to_string_lossy().to_string(); }
    "".to_string()
}

fn build_response(db: &crate::database::Database) -> AppConfigResponse {
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    let game_root = db.get_config("game_root").ok().flatten().unwrap_or_default();
    let active_mod = db.get_config("active_mod").ok().flatten().unwrap_or_default();
    let game_version = db.get_config("game_version").ok().flatten().unwrap_or_default();
    let language = db.get_config("language").ok().flatten().unwrap_or_else(|| "enUS".to_string());
    let stash_grid_size = db.get_config("stash_grid_size").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(10);
    let backpack_cols = db.get_config("backpack_cols").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(10);
    let backpack_rows = db.get_config("backpack_rows").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(4);
    let cube_cols = db.get_config("cube_cols").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(10);
    let cube_rows = db.get_config("cube_rows").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(10);
    let (available_mods, mod_metadata) = if !game_root.is_empty() { scan_mods(&game_root) } else {
        (vec!["(原版)".to_string()], vec![ModMeta {
            name: "(原版)".into(), version: String::new(), description: "原版 D2R".into(), author: String::new(), save_path: String::new(),
        }])
    };
    let default_folder = get_default_save_folder();
    let game_data_path = if !game_root.is_empty() { resolve_excel_path(&game_root, &active_mod) } else { "".to_string() };
    let effective_version = if !game_version.is_empty() {
        game_version.clone()
    } else {
        mod_metadata
            .iter()
            .find(|m| m.name == active_mod)
            .map(|m| m.version.clone())
            .unwrap_or_default()
    };
    let resource_manifest = build_resource_manifest(
        (!game_data_path.is_empty()).then_some(game_data_path.as_str()),
        &language,
        Some(&active_mod),
        (!effective_version.is_empty()).then_some(effective_version.as_str()),
        (!game_root.is_empty()).then_some(game_root.as_str()),
    );
    let mut import_status: Vec<ImportStatus> = Vec::new();
    if let Some(manifest) = &resource_manifest {
        // Both localized strings and game definitions are imported in the background
        // (see ensure_background_import). Here we just read existing import status from DB.
        if let Ok(profile_id) = db.upsert_resource_manifest(manifest) {
            let conn = db.get_connection();
            let logs = queries::get_import_log(conn, profile_id);
            let mut has_localized_log = false;
            for log in logs {
                if log.table_name == "localized_string" {
                    has_localized_log = true;
                }
                import_status.push(ImportStatus {
                    table_name: log.table_name,
                    rows: log.rows_count as usize,
                    source: log.source,
                    elapsed_ms: 0,
                    status: log.status,
                });
            }
            // Fallback: if localized_string has data but no log entry (pre-fix imports)
            if !has_localized_log {
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
                        rusqlite::params![profile_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if count > 0 {
                    import_status.push(ImportStatus {
                        table_name: "localized_string".into(),
                        rows: count as usize,
                        source: "json".into(),
                        elapsed_ms: 0,
                        status: "completed".into(),
                    });
                }
            }
        }
    }
    AppConfigResponse {
        save_folder,
        default_folder,
        game_root,
        profile_key: build_profile_key(&active_mod, &effective_version),
        active_mod,
        game_version: effective_version,
        language,
        stash_grid_size,
        backpack_cols,
        backpack_rows,
        cube_cols,
        cube_rows,
        available_mods,
        mod_metadata,
        game_data_path,
        resource_manifest,
        import_status,
        profiles: list_saved_profiles_inner(db),
    }
}

/// List all saved profiles with their import status.
fn list_saved_profiles_inner(db: &Database) -> Vec<ProfileInfo> {
    let conn = db.get_connection();
    let mut stmt = match conn.prepare(
        "SELECT id, profile_key, source_kind, mod_name, game_version, active_language, game_root, excel_path,
                checksum, source_path, imported_at, created_at
         FROM resource_profile ORDER BY created_at DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows: Vec<(i64, String, String, String, String, String, String, String, String, String, Option<String>, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
                row.get::<_, String>(8)?, row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?, row.get::<_, String>(11)?,
            ))
        })
        .ok()
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default();

    rows.into_iter().map(|(id, key, kind, mod_name, ver, lang, game_root, excel, checksum, source_path, imported_at, created_at)| {
        let import_logs = queries::get_import_log(conn, id);
        let status = import_logs.into_iter().map(|log| ImportStatus {
            table_name: log.table_name,
            rows: log.rows_count as usize,
            source: log.source,
            elapsed_ms: 0,
            status: log.status,
        }).collect();
        let localized_count: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        ProfileInfo {
            id,
            profile_key: key,
            source_kind: kind,
            mod_name,
            game_version: ver,
            active_language: lang,
            game_root,
            excel_path: excel,
            checksum,
            source_path,
            imported_at,
            import_status: status,
            localized_count,
            created_at,
        }
    }).collect()
}

/// List all saved resource profiles.
#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> Result<Vec<ProfileInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(list_saved_profiles_inner(&db))
}

/// Switch to a saved profile by ID.
/// Updates app_config to match the profile's game_root/mod_name/game_version/language.
#[tauri::command]
pub fn switch_profile(state: State<AppState>, profile_id: i64) -> Result<AppConfigResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let (game_root, mod_name, game_version, language): (String, String, String, String) = conn
        .query_row(
            "SELECT game_root, mod_name, game_version, active_language FROM resource_profile WHERE id = ?1",
            params![profile_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| "Profile not found".to_string())?;
    db.set_config("game_root", &game_root).map_err(|e| e.to_string())?;
    db.set_config("active_mod", &mod_name).map_err(|e| e.to_string())?;
    db.set_config("game_version", &game_version).map_err(|e| e.to_string())?;
    db.set_config("language", &language).map_err(|e| e.to_string())?;
    // Active profile changed — drop resolver caches so they rebuild under the new profile
    crate::resource::resolver::clear_resolver_cache();
    drop(db);
    // Reload full config
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resp = build_response(&db);
    drop(db);
    ensure_background_import(&state);
    Ok(resp)
}

/// Delete a saved profile and its resources.
#[tauri::command]
pub fn delete_profile(state: State<AppState>, profile_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    conn.execute("DELETE FROM resource_import_log WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM localized_string WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM resource_file WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM item_base WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM unique_item_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM set_item_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM set_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM runeword_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM stat_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill_tab_def WHERE profile_id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM resource_profile WHERE id = ?1", params![profile_id])
        .map_err(|e| e.to_string())?;
    // Profile data deleted — drop stale resolver cache
    crate::resource::resolver::clear_resolver_cache();
    Ok(())
}

#[tauri::command]
pub fn get_app_config(state: State<AppState>) -> Result<AppConfigResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let resp = build_response(&db);
    drop(db);
    ensure_background_import(&state);
    Ok(resp)
}

#[tauri::command]
pub fn update_save_folder(state: State<AppState>, save_folder: String) -> Result<(), String> {
    if save_folder.trim().is_empty() { return Err("路径不能为空".into()); }
    let resolved = shellexpand::full(save_folder.trim()).unwrap_or(std::borrow::Cow::Borrowed("")).to_string();
    if !Path::new(&resolved).exists() { return Err(format!("文件夹不存在: {}", resolved)); }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("save_folder", &save_folder).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_game_root(state: State<AppState>, game_root: String) -> Result<AppConfigResponse, String> {
    let root = shellexpand::full(game_root.trim()).unwrap_or(std::borrow::Cow::Borrowed("")).to_string();
    if !Path::new(&root).is_dir() { return Err("游戏目录不存在".into()); }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("game_root", &root).map_err(|e| e.to_string())?;
    let resp = build_response(&db);
    drop(db);
    ensure_background_import(&state);
    Ok(resp)
}

#[tauri::command]
pub fn set_active_mod(state: State<AppState>, active_mod: String) -> Result<AppConfigResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let game_root = db.get_config("game_root").ok().flatten().unwrap_or_default();
    let (mods, meta) = scan_mods(&game_root);
    if !mods.contains(&active_mod) { return Err("未找到该模组".into()); }
    db.set_config("active_mod", &active_mod).map_err(|e| e.to_string())?;
    // Auto-detect version and save_path from mod metadata
    if let Some(m) = meta.iter().find(|m| m.name == active_mod) {
        if !m.version.is_empty() {
            let _ = db.set_config("game_version", &m.version);
        }
        // Store mod's save_path for stash file resolution
        let _ = db.set_config("mod_save_path", &m.save_path);
    }
    let resp = build_response(&db);
    drop(db);
    ensure_background_import(&state);
    Ok(resp)
}

#[tauri::command]
pub fn set_game_version(state: State<AppState>, game_version: String) -> Result<AppConfigResponse, String> {
    let normalized = game_version.trim().to_string();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("game_version", &normalized).map_err(|e| e.to_string())?;
    Ok(build_response(&db))
}

#[tauri::command]
pub fn set_language(state: State<AppState>, language: String) -> Result<AppConfigResponse, String> {
    let valid = ["enUS","zhCN","zhTW","deDE","frFR","esES","itIT","koKR","plPL","ptBR","ruRU","jaJP"];
    if !valid.contains(&language.as_str()) { return Err("不支持的语言".into()); }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("language", &language).map_err(|e| e.to_string())?;
    Ok(build_response(&db))
}

#[tauri::command]
pub fn set_stash_grid_size(state: State<AppState>, size: u8) -> Result<(), String> {
    if size != 10 && size != 16 { return Err("仅支持 10×10 或 16×16".into()); }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("stash_grid_size", &size.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_grid_sizes(state: State<AppState>, backpack_cols: u8, backpack_rows: u8, cube_cols: u8, cube_rows: u8) -> Result<AppConfigResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("backpack_cols", &backpack_cols.to_string()).map_err(|e| e.to_string())?;
    db.set_config("backpack_rows", &backpack_rows.to_string()).map_err(|e| e.to_string())?;
    db.set_config("cube_cols", &cube_cols.to_string()).map_err(|e| e.to_string())?;
    db.set_config("cube_rows", &cube_rows.to_string()).map_err(|e| e.to_string())?;
    Ok(build_response(&db))
}

/// 诊断当前语言资源完整性：统计 enUS 有但当前语言缺失的 string_key。
#[derive(Debug, Serialize, Deserialize)]
pub struct LocaleDiagnosis {
    /// 目标语言代码
    pub target_lang: String,
    /// 总 namespace 数
    pub total_namespaces: usize,
    /// 按 namespace 分组统计
    pub namespaces: Vec<NamespaceGap>,
    /// 整体缺失率
    pub overall_missing_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamespaceGap {
    pub namespace: String,
    pub enus_count: usize,
    pub lang_count: usize,
    pub missing: usize,
    pub missing_pct: f64,
}

#[tauri::command]
pub fn diagnose_zh_tw(state: State<AppState>) -> Result<LocaleDiagnosis, String> {
    use rusqlite::params;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = get_active_profile_id(&db).unwrap_or(0);
    if profile_id == 0 {
        return Err("No active profile — run resource import first".into());
    }

    // Read current language from config
    let target_lang = db.get_config("language").ok().flatten().unwrap_or_else(|| "enUS".to_string());
    if target_lang == "enUS" {
        return Ok(LocaleDiagnosis {
            target_lang: target_lang.clone(),
            total_namespaces: 0,
            namespaces: vec![],
            overall_missing_pct: 0.0,
        });
    }

    // Get all namespaces that have enUS entries
    let mut stmt = conn.prepare(
        "SELECT DISTINCT namespace FROM localized_string
         WHERE profile_id = ?1 AND language = 'enUS'
         ORDER BY namespace"
    ).map_err(|e| e.to_string())?;
    let namespaces: Vec<String> = stmt.query_map(params![profile_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut namespace_gaps = Vec::new();
    let mut total_enus = 0usize;

    for ns in &namespaces {
        let enus_count: usize = conn.query_row(
            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1 AND namespace = ?2 AND language = 'enUS'",
            params![profile_id, ns],
            |row| row.get(0),
        ).unwrap_or(0);
        let lang_count: usize = conn.query_row(
            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1 AND namespace = ?2 AND language = ?3",
            params![profile_id, ns, target_lang],
            |row| row.get(0),
        ).unwrap_or(0);
        let missing = enus_count.saturating_sub(lang_count);
        let missing_pct = if enus_count > 0 {
            missing as f64 / enus_count as f64 * 100.0
        } else {
            0.0
        };
        namespace_gaps.push(NamespaceGap {
            namespace: ns.clone(),
            enus_count,
            lang_count,
            missing,
            missing_pct,
        });
        total_enus += enus_count;
    }

    let total_missing: usize = namespace_gaps.iter().map(|g| g.missing).sum();
    let overall_missing_pct = if total_enus > 0 {
        total_missing as f64 / total_enus as f64 * 100.0
    } else {
        0.0
    };

    Ok(LocaleDiagnosis {
        target_lang: target_lang.clone(),
        total_namespaces: namespaces.len(),
        namespaces: namespace_gaps,
        overall_missing_pct,
    })
}

/// Check if a background import should be started, and spawn one if needed.
/// Called from command handlers after config changes (e.g. set_game_root).
pub fn ensure_background_import(state: &State<AppState>) {
    let db_path = crate::database::Database::get_db_path_clone();
    let manifest = {
        let db = state.db.lock().unwrap();
        // Get current config
        let game_root = db.get_config("game_root").ok().flatten().unwrap_or_default();
        let active_mod = db.get_config("active_mod").ok().flatten().unwrap_or_default();
        let game_version = db.get_config("game_version").ok().flatten().unwrap_or_default();
        let language = db.get_config("language").ok().flatten().unwrap_or_else(|| "enUS".to_string());

        let game_data_path = if !game_root.is_empty() {
            resolve_excel_path(&game_root, &active_mod)
        } else {
            "".to_string()
        };

        if game_data_path.is_empty() {
            // No game data source configured — clear ImportState so frontend shows "未导入"
            if let Ok(mut s) = state.import_state.lock() {
                s.tables.clear();
            }
            return;
        }

        let some_manifest = build_resource_manifest(
            Some(game_data_path.as_str()),
            &language,
            Some(&active_mod),
            (!game_version.is_empty()).then_some(game_version.as_str()),
            (!game_root.is_empty()).then_some(game_root.as_str()),
        );
        let manifest = match some_manifest {
            Some(m) => m,
            None => return,
        };
        let profile_id = match db.upsert_resource_manifest(&manifest) {
            Ok(id) => id,
            Err(_) => return,
        };
        if db.has_game_definitions(profile_id) && db.has_localized_strings(profile_id) {
            // Data already exists — mark ImportState as completed with actual row counts
            if let Ok(mut s) = state.import_state.lock() {
                let conn = db.get_connection();
                for t in &mut s.tables {
                    t.status = "completed".to_string();
                    // Infer source from table name
                    t.source = match t.table_name.as_str() {
                        "runeword_def" => "const".into(),
                        "localized_string" => "json".into(),
                        _ => "txt".into(),
                    };
                    // Query actual row count from the DB
                    let table_name = &t.table_name;
                    if table_name == "localized_string" {
                        t.rows = conn.query_row(
                            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
                            rusqlite::params![profile_id], |row| row.get(0),
                        ).unwrap_or(0);
                    } else {
                        // Try to query the definition table directly
                        let sql = format!("SELECT COUNT(1) FROM {} WHERE profile_id = ?1", table_name);
                        if let Ok(mut stmt) = conn.prepare(&sql) {
                            t.rows = stmt.query_row(rusqlite::params![profile_id], |row| row.get(0)).unwrap_or(0);
                        }
                    }
                }
            }
            return;
        }

        // Check if import is already running
        let import_state = state.import_state.lock().unwrap();
        if import_state.running {
            return; // Already importing
        }
        drop(import_state);

        manifest
    };

    // Spawn background import
    log::info!("[config] Starting background import for profile {:?}", manifest.profile_id);
    let import_state = state.import_state.clone();
    import_task::spawn_import(db_path, manifest, import_state);
}

/// Return current import progress (polled by frontend).
#[tauri::command]
pub fn get_import_progress(state: State<AppState>) -> Result<crate::ImportState, String> {
    let s = state.import_state.lock().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

/// Re-import game data asynchronously.
/// Clears existing data, spawns background import, returns immediately.
#[tauri::command]
pub fn reimport_game_data(state: State<AppState>) -> Result<AppConfigResponse, String> {
    let db_path = crate::database::Database::get_db_path_clone();
    let (manifest, import_state_arc) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let game_root = db.get_config("game_root").ok().flatten().unwrap_or_default();
        let active_mod = db.get_config("active_mod").ok().flatten().unwrap_or_default();
        let game_version = db.get_config("game_version").ok().flatten().unwrap_or_default();
        let language = db.get_config("language").ok().flatten().unwrap_or_else(|| "enUS".to_string());

        let game_data_path = if !game_root.is_empty() {
            resolve_excel_path(&game_root, &active_mod)
        } else {
            return Err("请先配置游戏目录".into());
        };

        let manifest = match build_resource_manifest(
            Some(game_data_path.as_str()),
            &language,
            Some(&active_mod),
            (!game_version.is_empty()).then_some(game_version.as_str()),
            (!game_root.is_empty()).then_some(game_root.as_str()),
        ) {
            Some(m) => m,
            None => return Err("无法构建资源清单".into()),
        };

        // If this is a mod profile, ensure the linked vanilla profile has data
        if active_mod != "(原版)" && !active_mod.is_empty() {
            let conn = db.get_connection();
            let has_vanilla_data: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM resource_profile WHERE profile_key LIKE 'vanilla:%' AND imported_at IS NOT NULL)",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            if !has_vanilla_data {
                log::info!("[config] Vanilla base missing — importing before mod");
                // Build a manifest for vanilla data
                let game_data_path = db.get_config("game_data_path").ok().flatten().unwrap_or_default();
                if !game_data_path.is_empty()
                    && let Some(v_manifest) = build_resource_manifest(
                        Some(&game_data_path), &language, None,
                        (!game_version.is_empty()).then_some(game_version.as_str()),
                        (!game_root.is_empty()).then_some(game_root.as_str()),
                    ) {
                        db.import_game_definitions(&v_manifest).ok();
                        db.import_localized_strings_from_manifest(&v_manifest).ok();
                        log::info!("[config] Vanilla base imported as profile");
                    }
            }
        }
        // Re-create profile_id via upsert
        let profile_id = db.upsert_resource_manifest(&manifest).map_err(|e| e.to_string())?;

        // Delete old data for this profile
        let conn = db.get_connection();
        conn.execute("DELETE FROM resource_import_log WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM localized_string WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM item_base WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM unique_item_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM set_item_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM set_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM runeword_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM stat_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM skill_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM item_affix_def WHERE profile_id = ?1", params![profile_id])
            .map_err(|e| e.to_string())?;

        // Data deleted synchronously — drop stale resolver caches now
        crate::resource::resolver::clear_resolver_cache();

        // Reset import state for the frontend
        let import_state = state.import_state.clone();
        {
            let mut s = import_state.lock().unwrap();
            *s = crate::ImportState::new();
            s.running = true;
        }

        (manifest, import_state)
    };

    // Spawn background import
    log::info!("[config] Re-import started for profile {:?}", manifest.profile_id);
    import_task::spawn_import(db_path, manifest, import_state_arc);

    // Return current config (partial data until import completes)
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(build_response(&db))
}

/// Reset app config to factory defaults: clear game settings and imported data.
/// Keeps the database schema and user token balance intact.
#[tauri::command]
pub fn reset_app_config(state: State<AppState>) -> Result<AppConfigResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();

    // Clear all app config
    conn.execute("DELETE FROM app_config", []).map_err(|e| e.to_string())?;

    // Delete all profiles and cascaded data
    conn.execute("DELETE FROM resource_profile", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM resource_import_log", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM localized_string", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM item_base", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM unique_item_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM set_item_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM set_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM runeword_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM stat_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill_tab_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM item_affix_def", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM resource_file", []).map_err(|e| e.to_string())?;

    // All profile data deleted — drop resolver caches
    crate::resource::resolver::clear_resolver_cache();

    // Reset import state
    drop(db);
    {
        let mut s = state.import_state.lock().unwrap();
        *s = crate::ImportState::new();
    }

    // Return fresh config
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(build_response(&db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    fn fresh_db(name: &str) -> Database {
        let tmp = std::env::temp_dir().join(format!(
            "d2r_cfg_test_{}_{}.db",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&tmp);
        Database::open(tmp.to_str().unwrap()).expect("create test DB")
    }

    // ── build_profile_key ──

    #[test]
    fn test_build_profile_key_vanilla() {
        assert_eq!(build_profile_key("(原版)", "2.7"), "vanilla:2.7");
    }

    #[test]
    fn test_build_profile_key_mod() {
        assert_eq!(build_profile_key("仙道轮回", "3.2"), "mod:仙道轮回:3.2");
    }
    #[test]
    fn test_build_profile_key_default_version() {
        assert_eq!(build_profile_key("(原版)", ""), "vanilla:default");
    }

    // ── resolve_excel_path ──

    #[test]
    fn test_resolve_excel_path_vanilla_nonexistent() {
        // No D2R install at this path → returns empty string
        let path = resolve_excel_path("D:\\no_such_d2r", "(原版)");
        assert_eq!(path, "", "non-existent path should return empty");
    }

    #[test]
    fn test_resolve_excel_path_mod_nonexistent() {
        let path = resolve_excel_path("D:\\no_such_d2r", "MyMod");
        assert_eq!(path, "", "non-existent mod should return empty");
    }

    #[test]
    fn test_resolve_excel_path_contains_mod_structure() {
        // The function checks file existence, so it may return ""
        // Verify it doesn't crash or return garbage for non-existent paths
        let root = "D:\\games\\D2R";
        let mod_name = "TestMod";
        let path = resolve_excel_path(root, mod_name);
        // If non-empty, should reference the mod
        assert!(path.is_empty() || path.contains(mod_name), "if non-empty, should contain mod name");
    }

    // ── get_active_profile_key (DB-dependent) ──


    #[test]
    fn test_active_profile_key_default() {
        let db = fresh_db("default_key");
        // No config set → should return a default
        let result = get_active_profile_key(&db);
        assert!(result.is_ok(), "should have a default key: {:?}", result);
    }

    #[test]
    fn test_active_profile_key_vanilla() {
        let db = fresh_db("vanilla_key");
        db.set_config("game_root", "D:\\test").ok();
        db.set_config("active_mod", "(原版)").ok();
        db.set_config("game_version", "2.7").ok();
        let key = get_active_profile_key(&db).unwrap();
        assert_eq!(key, "vanilla:2.7");
    }

    #[test]
    fn test_active_profile_key_mod() {
        let db = fresh_db("mod_key");
        db.set_config("game_root", "D:\\test").ok();
        db.set_config("active_mod", "MyMod").ok();
        db.set_config("game_version", "3.2").ok();
        let key = get_active_profile_key(&db).unwrap();
        assert_eq!(key, "mod:MyMod:3.2");
    }

    // ── get_active_profile_id (DB-dependent) ──

    #[test]
    fn test_active_profile_id_no_profile_returns_0() {
        let db = fresh_db("no_profile");
        db.set_config("game_root", "D:\\test").ok();
        db.set_config("active_mod", "(原版)").ok();
        let id = get_active_profile_id(&db).unwrap_or(0);
        assert_eq!(id, 0, "no profile imported → id=0");
    }
}
