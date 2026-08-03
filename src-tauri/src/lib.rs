//! D2R Marketplace Tauri 库入口。
//!
//! - `core/` 框架无关基础设施（位流、编码、统一 Result、版本枚举）
//! - `database/` SQLite 持久层
//! - `data/` 游戏数据（物品、符文之语、词缀、中英文名称表）
//! - `commands/` Tauri IPC 命令处理器
//! - `protocol/` d2i/d2s 位流解析器
//! - `market/` 经济规则层
//! - `resource/` 数据导入/查询/tooltip 资源层
//! - `services/` 业务服务层
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::needless_borrows_for_generic_args,
    clippy::unnecessary_cast,
    clippy::needless_range_loop,
    clippy::clone_on_copy,
    clippy::manual_div_ceil,
    clippy::manual_is_multiple_of,
    clippy::manual_checked_ops,
    clippy::manual_range_contains,
    clippy::explicit_counter_loop,
    clippy::comparison_chain,
    clippy::vec_init_then_push,
    clippy::if_same_then_else,
    clippy::manual_clamp,
    clippy::possible_missing_else,
    clippy::unnecessary_sort_by,
    clippy::doc_lazy_continuation,
    clippy::manual_strip,
    clippy::empty_line_after_doc_comments,
)]

pub mod commands;
pub mod core;
pub mod database;
pub mod data;
pub mod io;
pub mod market;
pub mod protocol;
pub mod resource;
pub mod services;

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Per-table import progress shown to the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableImportProgress {
    pub table_name: String,
    pub rows: usize,
    pub elapsed_ms: u64,
    pub status: String, // "pending" | "importing" | "completed" | "error"
    #[serde(default)]
    pub source: String,
}

/// Background import task state, polled by the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportState {
    pub running: bool,
    pub tables: Vec<TableImportProgress>,
    pub error: Option<String>,
}

impl Default for ImportState {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportState {
    pub fn new() -> Self {
        let table_names = [
            "item_base", "unique_item_def", "set_item_def",
            "runeword_def", "stat_def", "skill_def",
            "item_affix_def", "localized_string",
        ];
        Self {
            running: false,
            tables: table_names.into_iter().map(|n| TableImportProgress {
                table_name: n.to_string(),
                rows: 0,
                elapsed_ms: 0,
                status: "pending".to_string(),
                source: String::new(),
            }).collect(),
            error: None,
        }
    }
}

/// Tauri 全局状态：持有数据库互斥锁和后台导入状态。
pub struct AppState {
    pub db: Mutex<database::Database>,
    pub import_state: Arc<Mutex<ImportState>>,
}

/// Tauri 应用入口。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志（默认 INFO，使用本地时区）
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
            writeln!(buf, "[{} {} {}] {}", ts, record.level(), record.target(), record.args())
        })
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db = database::Database::init().map_err(|e| {
                log::error!("Failed to init database: {}", e);
                Box::new(std::io::Error::other(e.to_string()))
                    as Box<dyn std::error::Error>
            })?;
            app.manage(AppState {
                db: Mutex::new(db),
                import_state: Arc::new(Mutex::new(ImportState::new())),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::balance::get_balance,
            commands::config::get_app_config,
            commands::config::update_save_folder,
            commands::config::set_game_root,
            commands::config::set_active_mod,
            commands::config::set_game_version,
            commands::config::set_language,
            commands::config::set_stash_grid_size,
            commands::config::set_grid_sizes,
            commands::config::list_profiles,
            commands::config::switch_profile,
            commands::config::reimport_game_data,
            commands::config::get_import_progress,
            commands::config::reset_app_config,
            commands::config::delete_profile,
            commands::build::get_build_recommendations,
            commands::config::diagnose_zh_tw,
            commands::runeword_calc::find_runewords,
            commands::runeword_calc::get_runeword_context,
            commands::grail::get_grail,
            commands::grail::toggle_grail,
            commands::stash::read_stash,
            commands::crafted::get_crafted_context,

            commands::stash::create_stash_backup,
            commands::stash::list_backups,
            commands::stash::restore_backup,
            commands::stash::list_auto_backups,
            commands::stash::restore_auto_backup,
            commands::stash::cleanup_auto_backups,
            commands::stash::delete_backup,
            commands::stash::archive_old_auto_backups,
            commands::stash::get_auto_backup_retention,
            commands::stash::set_auto_backup_retention,
            commands::stash::auto_save_stash,
            commands::stash::get_auto_save_info,
            commands::stash::restore_auto_save,
            commands::stash::list_safety_backups,
            commands::marketplace::buy_item,
            commands::marketplace::cancel_listing,
            commands::marketplace::get_listed_items,
            commands::marketplace::get_price_suggestion,
            commands::marketplace::list_item,
            commands::marketplace::update_listing_price,
            commands::warehouse::warehouse_list,
            commands::warehouse::warehouse_list_pages,
            commands::warehouse::warehouse_list_by_page,
            commands::warehouse::warehouse_update_meta,
            commands::warehouse::warehouse_rename_page,
            commands::warehouse::warehouse_delete_page,
            commands::warehouse::warehouse_deposit,
            commands::warehouse::warehouse_withdraw,
            commands::warehouse::warehouse_search,
            commands::warehouse::warehouse_backfill_dims,
            commands::warehouse::warehouse_remove,
            commands::warehouse::warehouse_set_code_default,
            commands::warehouse::warehouse_clear_code_default,
            commands::warehouse::warehouse_set_item_default,
            commands::warehouse::warehouse_clear_item_default,
            commands::warehouse::warehouse_resolve_default,
            commands::character::read_character_info,
            commands::character::list_characters,
            commands::character::list_characters_brief,
            commands::character::extract_character_equipment,
            commands::character::load_character_background,
            commands::character::get_localized_skill_texts,
            commands::history::get_transactions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
