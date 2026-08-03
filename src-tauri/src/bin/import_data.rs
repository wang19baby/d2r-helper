//! One-shot import: loads base game + mod data from cascview + mods dirs into DB.
//!
//! Strategy:
//!   1. Import base game (cascview) → vanilla profile
//!   2. For each installed mod with excel files → mod profile
//!
//! Mod profiles store the COMPLETE mod data (base + overrides merged by D2RMM).
//! The resolver cascades: mod_profile -> vanilla_profile for unresolved lookups.
//!
//! Run: cargo run --bin import_data

use d2r_marketplace_lib::protocol::d2i::legacy::resource_manifest::{
    build_resource_manifest, ResourceManifest,
};
use d2r_marketplace_lib::resource::importer::ResourceImporter;
use std::path::Path;

fn main() {
    let game_data_path = r"D:\dev\d2r\cascview_cn\x64\Work\data\data";
    let game_root = r"D:\dev\d2r\cascview_cn";
    let d2r_install_root = r"D:\personal\games\Diablo II Resurrected";

    let localappdata = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
    let db_path = std::path::Path::new(&localappdata)
        .join("D2RMarketplace").join("database").join("d2r_marketplace.db");
    if !db_path.exists() {
        eprintln!("Database not found at: {:?}", db_path);
        eprintln!("Run the Tauri app once to initialize the database.");
        std::process::exit(1);
    }
    println!("DB: {:?}", db_path);

    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open DB");
    let _ = conn.execute_batch("PRAGMA busy_timeout = 30000;");

    // ═══════════════════════════════════════════════
    // Step 1: Import base game (vanilla)
    // ═══════════════════════════════════════════════
    println!("\n═════ BASE GAME IMPORT ═════");
    println!("Data: {}", game_data_path);

    let vanilla_manifest = build_resource_manifest(
        Some(game_data_path), "zhCN", None, None, Some(game_root),
    ).expect("Failed to build vanilla resource manifest");

    let vanilla_id = upsert_profile(&conn, &vanilla_manifest);
    clear_profile_data(&conn, vanilla_id);
    println!("Vanilla profile: {} (key: {})", vanilla_id, vanilla_manifest.profile_id);

    // TXT import
    println!("\n  --- TXT definitions ---");
    let importer = ResourceImporter::new(&conn, vanilla_id);
    for r in importer.import_all(&vanilla_manifest.excel_path) {
        println!("    {:25}: {} rows ({})", r.table, r.rows, r.source);
    }

    // JSON strings import
    println!("\n  --- JSON strings ---");
    let mut total = 0usize;
    for file in &vanilla_manifest.json_files {
        if !file.exists {
            println!("    SKIP {}", file.path);
            continue;
        }
        let ns = normalize_namespace(&file.role);
        let count = import_json(&conn, vanilla_id, &file.path, ns).unwrap_or(0);
        println!("    {:25}: {} rows", ns, count);
        total += count;
    }
    println!("    --- total: {} strings", total);

    mark_imported(&conn, vanilla_id);

    // ═══════════════════════════════════════════════
    // Step 2: Import installed mods
    // ═══════════════════════════════════════════════
    println!("\n\n═════ MOD IMPORT ═════");

    let mods_data = scan_installed_mods(d2r_install_root);
    if mods_data.is_empty() {
        println!("No mods found in {}", Path::new(d2r_install_root).join("mods").display());
    }

    for info in &mods_data {
        println!("\n  Mod: {} ({})", info.name, info.version.as_deref().unwrap_or("unknown"));

        if !info.has_excel {
            println!("    SKIP: no excel directory found");
            continue;
        }

        let mod_manifest = build_resource_manifest(
            Some(&info.excel_path), "zhCN",
            Some(&info.name), Some("3.2-92777"), Some(game_root),
        ).expect("Failed to build mod resource manifest");

        let mod_profile_id = upsert_profile(&conn, &mod_manifest);
        clear_profile_data(&conn, mod_profile_id);
        println!("    Profile: {} (key: {})", mod_profile_id, mod_manifest.profile_id);

        // Link mod -> vanilla for fallback
        let _ = conn.execute(
            "UPDATE resource_profile SET vanilla_profile_id = ?1 WHERE id = ?2",
            rusqlite::params![vanilla_id, mod_profile_id],
        );

        // TXT import from mod's excel
        println!("\n    --- TXT definitions ---");
        let mod_importer = ResourceImporter::new(&conn, mod_profile_id);
        for r in mod_importer.import_all(&mod_manifest.excel_path) {
            let note = if r.source == "hardcoded" {
                " (hardcoded fallback)"
            } else {
                ""
            };
            println!("      {:25}: {} rows ({}){}", r.table, r.rows, r.source, note);
        }

        // JSON strings import
        println!("\n    --- JSON strings ---");
        let mut mod_total = 0usize;
        for file in &mod_manifest.json_files {
            if !file.exists {
                println!("      SKIP {}", file.path);
                continue;
            }
            let ns = normalize_namespace(&file.role);
            let count = import_json(&conn, mod_profile_id, &file.path, ns).unwrap_or(0);
            println!("      {:25}: {} rows", ns, count);
            mod_total += count;
        }
        println!("      --- total: {} strings", mod_total);

        mark_imported(&conn, mod_profile_id);
        println!("    ✅ {} imported", info.name);
    }

    // Verify all profiles
    println!("\n\n═════ VERIFICATION ═════");
    let mut all_profiles = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id FROM resource_profile ORDER BY id")
        && let Ok(rows) = stmt.query_map([], |r| r.get::<_, i64>(0)) {
            for row in rows.flatten() {
                all_profiles.push(row);
            }
        }
    for pid in all_profiles {
        println!();
        verify(&conn, pid);
    }

    println!("\n═════ DONE ═════");
}

// ── Mod detection ─────────────────────────────────

struct InstalledMod {
    name: String,
    excel_path: String,
    has_excel: bool,
    version: Option<String>,
}

fn scan_installed_mods(d2r_install_root: &str) -> Vec<InstalledMod> {
    let mods_dir = Path::new(d2r_install_root).join("mods");
    if !mods_dir.is_dir() {
        return vec![];
    }

    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let base = mods_dir.join(&name);
            let m = format!("{}.mpq", name);
            let candidate = base.join(&m).join("data").join("global").join("excel");
            let has_excel = candidate.is_dir();
            let version = read_mod_version(&base.join("modinfo.json"))
                .or_else(|| read_mod_version(&base.join("mod.json")));

            result.push(InstalledMod {
                name,
                excel_path: candidate.to_string_lossy().to_string(),
                has_excel,
                version,
            });
        }
    }
    result
}

fn read_mod_version(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let cleaned: String = content
        .lines()
        .filter(|l| !l.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let v: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    v.get("version").and_then(|v| v.as_str()).map(|s| s.to_string())
}

// ── Profile helpers ────────────────────────────────

fn upsert_profile(conn: &rusqlite::Connection, manifest: &ResourceManifest) -> i64 {
    let pk = &manifest.profile_id;
    if let Ok(id) = conn.query_row(
        "SELECT id FROM resource_profile WHERE profile_key = ?1",
        rusqlite::params![pk],
        |row| row.get(0),
    ) {
        let _ = conn.execute(
            "UPDATE resource_profile SET mod_name=?1, game_version=?2, source_kind=?3, active_language=?4, updated_at=CURRENT_TIMESTAMP WHERE id=?5",
            rusqlite::params![manifest.mod_name, manifest.game_version, manifest.source_kind, manifest.active_language, id],
        );
        id
    } else {
        let _ = conn.execute(
            "INSERT INTO resource_profile (profile_key, source_kind, game_version, mod_name, active_language) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![pk, manifest.source_kind, manifest.game_version, manifest.mod_name, manifest.active_language],
        );
        conn.last_insert_rowid()
    }
}

fn clear_profile_data(conn: &rusqlite::Connection, profile_id: i64) {
    for t in &[
        "localized_string", "item_base", "unique_item_def", "set_item_def",
        "set_def", "runeword_def", "stat_def", "skill_def", "skill_tab_def", "item_affix_def",
    ] {
        conn.execute(
            &format!("DELETE FROM {} WHERE profile_id = ?1", t),
            rusqlite::params![profile_id],
        ).ok();
    }
    conn.execute(
        "DELETE FROM resource_import_log WHERE profile_id = ?1",
        rusqlite::params![profile_id],
    ).ok();
}

fn mark_imported(conn: &rusqlite::Connection, profile_id: i64) {
    let _ = conn.execute(
        "UPDATE resource_profile SET imported_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        rusqlite::params![profile_id],
    );
}

// ── JSON import ────────────────────────────────────

fn normalize_namespace(role: &str) -> &str {
    match role {
        "item_names" | "legacy_item_names" => "item_names",
        "item_runes" | "legacy_item_runes" => "item_runes",
        "item_gems" | "legacy_item_gems" => "item_gems",
        _ => role,
    }
}

fn import_json(
    conn: &rusqlite::Connection,
    profile_id: i64,
    path: &str,
    namespace: &str,
) -> Result<usize, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("Read: {}", e))?;
    let raw = if let Some(s) = raw.strip_prefix('\u{FEFF}') { s.to_string() } else { raw };
    let cleaned: Vec<&str> = raw.lines()
        .map(|l| l.trim_start())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();
    let input = if cleaned.is_empty() { &raw } else { &cleaned.join("\n") };
    let entries: Vec<serde_json::Value> = serde_json::from_str(input).map_err(|e| format!("JSON: {}", e))?;

    conn.execute_batch("BEGIN").map_err(|e| format!("tx: {}", e))?;
    let mut count = 0usize;
    for entry in &entries {
        let obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };
        let key = match obj.get("Key").or_else(|| obj.get("key")).and_then(|v| v.as_str()) {
            Some(k) => k.to_lowercase(),
            None => continue,
        };
        for (lang, val) in obj.iter() {
            if lang == "id" || lang == "Key" || lang == "key" { continue; }
            let text = match val.as_str() {
                Some(s) if !s.is_empty() => s,
                _ => continue,
            };
            conn.execute(
                "INSERT OR REPLACE INTO localized_string
                 (profile_id, namespace, string_key, language, text_value, source_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![profile_id, namespace, key, lang, text, path],
            ).map_err(|e| format!("Insert: {}", e))?;
            count += 1;
        }
    }
    conn.execute_batch("COMMIT").map_err(|e| format!("commit: {}", e))?;
    Ok(count)
}

// ── Verification ───────────────────────────────────

fn verify(conn: &rusqlite::Connection, profile_id: i64) {
    let profile_key: String = conn.query_row(
        "SELECT profile_key FROM resource_profile WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| row.get(0),
    ).unwrap_or_default();
    println!("  Profile {} ({})", profile_id, profile_key);
    for name in &[
        "item_base", "unique_item_def", "set_item_def", "set_def",
        "runeword_def", "stat_def", "skill_def", "item_affix_def",
    ] {
        let c: i64 = conn.query_row(
            &format!("SELECT COUNT(1) FROM {} WHERE profile_id = ?1", name),
            rusqlite::params![profile_id],
            |row| row.get(0),
        ).unwrap_or(0);
        println!("    {:20}: {}", name, c);
    }
    let lc: i64 = conn.query_row(
        "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
        rusqlite::params![profile_id],
        |row| row.get(0),
    ).unwrap_or(0);
    println!("    {:20}: {}", "localized_string", lc);
}
