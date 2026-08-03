//! Background import task — runs TXT/JSON → SQLite import off the main thread.
//!
//! Opens its own SQLite connection so it never blocks the main DB mutex.
//! Progress is published through a shared `ImportState` for frontend polling.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use rusqlite::Connection;

use crate::resource::importer::{ResourceImporter, ImportResult};
use crate::protocol::d2i::legacy::resource_manifest::ResourceManifest;
use crate::ImportState;

/// Spawn a background thread that imports game definitions + localized strings.
///
/// The thread opens its own SQLite connection so the main DB lock is never held.
/// `import_state` is shared with `AppState` so the frontend can poll progress.
pub fn spawn_import(
    db_path: std::path::PathBuf,
    manifest: ResourceManifest,
    import_state: Arc<Mutex<ImportState>>,
) {
    std::thread::spawn(move || {
        let start = Instant::now();

        // Mark running
        set_running(&import_state, true, None);

        // Open our own connection (with busy timeout to avoid SQLITE_BUSY)
        let conn = match Connection::open(&db_path) {
            Ok(c) => {
                let _ = c.execute_batch("PRAGMA busy_timeout = 10000;");
                c
            }
            Err(e) => {
                set_running(&import_state, false, Some(format!("Failed to open DB: {}", e)));
                return;
            }
        };

        // Upsert the resource profile and get profile_id
        let profile_id = match upsert_profile(&conn, &manifest) {
            Ok(id) => id,
            Err(e) => {
                set_running(&import_state, false, Some(e));
                return;
            }
        };

        let importer = ResourceImporter::new(&conn, profile_id);
        let excel_path = &manifest.excel_path;

        // Run each import step sequentially, updating progress after each
        // Each step is wrapped in its own transaction for performance (bulk INSERT)
        run_import_step(&import_state, &importer, &conn, excel_path, "item_base", |i, p| i.import_item_base(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "unique_item_def", |i, p| i.import_unique_items(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "set_item_def", |i, p| i.import_set_items(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "runeword_def", |i, p| i.import_runewords(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "stat_def", |i, p| i.import_stat_def(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "skill_def", |i, p| i.import_skill_def(p));
        run_import_step(&import_state, &importer, &conn, excel_path, "item_affix_def", |i, p| i.import_item_affix(p));

        // Import localized strings from JSON files
        {
            update_table_state(&import_state, "localized_string", 0, 0, "importing", "json");
            let table_start = Instant::now();
            let _ = conn.execute_batch("BEGIN");
            match import_localized_strings(&conn, profile_id, &manifest) {
                Ok(count) => {
                    let _ = conn.execute_batch("COMMIT");
                    let elapsed = table_start.elapsed();
                    update_table_state(&import_state, "localized_string", count, elapsed.as_millis() as u64, "completed", "json");
                    let _ = conn.execute(
                        "DELETE FROM resource_import_log WHERE profile_id = ?1 AND table_name = 'localized_string'",
                        rusqlite::params![profile_id],
                    );
                    let _ = conn.execute(
                        "INSERT INTO resource_import_log (profile_id, table_name, rows_count, source, completed_at, status)
                         VALUES (?1, 'localized_string', ?2, 'json', CURRENT_TIMESTAMP, 'completed')",
                        rusqlite::params![profile_id, count],
                    );
                    log::info!("[bg_import] localized_string: {} rows in {:.2}s", count, elapsed.as_secs_f64());
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    let elapsed = table_start.elapsed();
                    update_table_state(&import_state, "localized_string", 0, elapsed.as_millis() as u64, "error", "json");
                    log::error!("[bg_import] localized_string failed: {}", e);
                }
            }
        }

        // Mark done — update imported_at to actual completion time
        set_running(&import_state, false, None);
        let _ = conn.execute(
            "UPDATE resource_profile SET imported_at = CURRENT_TIMESTAMP WHERE id = ?1",
            rusqlite::params![profile_id],
        );
        log::info!("[bg_import] All imports completed in {:.2}s", start.elapsed().as_secs_f64());
        // P0: imported data changed — drop stale per-profile resolver caches
        crate::resource::resolver::clear_resolver_cache();
    });
}

fn run_import_step(
    import_state: &Arc<Mutex<ImportState>>,
    importer: &ResourceImporter,
    conn: &Connection,
    excel_path: &str,
    name: &str,
    import_fn: fn(&ResourceImporter, &str) -> ImportResult,
) {
    update_table_state(import_state, name, 0, 0, "importing", "");
    let table_start = Instant::now();
    // Wrap each table import in a transaction to avoid per-row autocommit overhead
    let _ = conn.execute_batch("BEGIN");
    let result = import_fn(importer, excel_path);
    let _ = conn.execute_batch("COMMIT");
    let elapsed = table_start.elapsed();

    update_table_state(
        import_state,
        name,
        result.rows,
        elapsed.as_millis() as u64,
        if result.table.is_empty() { "error" } else { "completed" },
        &result.source,
    );
    log::info!("[bg_import] {}: {} rows from {} in {:.2}s", name, result.rows, result.source, elapsed.as_secs_f64());
}

fn set_running(import_state: &Arc<Mutex<ImportState>>, running: bool, error: Option<String>) {
    if let Ok(mut state) = import_state.lock() {
        state.running = running;
        if let Some(e) = error {
            state.error = Some(e);
        }
    }
}

fn update_table_state(
    import_state: &Arc<Mutex<ImportState>>,
    table_name: &str,
    rows: usize,
    elapsed_ms: u64,
    status: &str,
    source: &str,
) {
    if let Ok(mut state) = import_state.lock()
        && let Some(t) = state.tables.iter_mut().find(|t| t.table_name == table_name) {
            t.rows = rows;
            t.elapsed_ms = elapsed_ms;
            t.source = source.to_string();
            t.status = status.to_string();
        }
}

fn upsert_profile(conn: &Connection, manifest: &ResourceManifest) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO resource_profile (
            profile_key, source_kind, mod_name, game_version, active_language,
            game_root, excel_path, strings_path, strings_legacy_path,
            vanilla_profile_id, checksum, source_path, imported_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(profile_key) DO UPDATE SET
            game_root=excluded.game_root, excel_path=excluded.excel_path,
            strings_path=excluded.strings_path, strings_legacy_path=excluded.strings_legacy_path,
            source_path=excluded.source_path, updated_at=CURRENT_TIMESTAMP", 
        rusqlite::params![
            manifest.profile_id,
            manifest.source_kind,
            manifest.mod_name,
            manifest.game_version,
            manifest.active_language,
            manifest.game_root,
            manifest.excel_path,
            manifest.strings_path,
            manifest.strings_legacy_path,
            manifest.vanilla_profile_id,
            manifest.checksum,
            manifest.source_path,
        ],
    ).map_err(|e| format!("Failed to upsert profile: {}", e))?;

    let id: i64 = conn.query_row(
        "SELECT id FROM resource_profile WHERE profile_key = ?1",
        rusqlite::params![manifest.profile_id],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to get profile id: {}", e))?;

    Ok(id)
}

fn import_localized_strings(conn: &Connection, profile_id: i64, manifest: &ResourceManifest) -> Result<usize, String> {
    // Check if already imported
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
            rusqlite::params![profile_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if existing > 0 {
        return Ok(existing as usize);
    }

    let mut inserted = 0usize;
    for file in &manifest.json_files {
        let count = crate::database::db::import_string_file(conn, profile_id, file)
            .map_err(|e| format!("Failed to import strings from {}: {}", file.path, e))?;
        inserted += count;
    }
    Ok(inserted)
}
