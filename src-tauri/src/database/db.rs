use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

use super::models::*;
use crate::protocol::d2i::legacy::resource_manifest::{ResourceFileInfo, ResourceManifest};
use crate::resource::importer::ResourceImporter;

/// Database wrapper for the D2R Marketplace SQLite database
pub struct Database {
    conn: Connection,
}


fn clean_d2r_json(raw: &str) -> String {
    let content = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let cleaned: Vec<String> = content
        .lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") {
                String::new()
            } else if t.starts_with("[//") {
                "[".to_string()
            } else {
                l.to_string()
            }
        })
        .filter(|l| !l.is_empty())
        .collect();
    if cleaned.is_empty() {
        content.to_string()
    } else {
        cleaned.join("\n")
    }
}

pub(crate) fn import_string_file(conn: &Connection, profile_id: i64, file: &ResourceFileInfo) -> Result<usize> {
    if !file.exists || file.file_type != "json" {
        return Ok(0);
    }
    let raw = match std::fs::read_to_string(&file.path) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[import_string_file] cannot read {}: {}", file.path, e);
            return Ok(0);
        }
    };
    let cleaned = clean_d2r_json(&raw);
    let entries: Vec<serde_json::Value> = match serde_json::from_str(&cleaned) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[import_string_file] JSON parse error in {}: {}", file.path, e);
            return Ok(0);
        }
    };
    let mut inserted = 0usize;
    for entry in entries {
        let Some(key) = entry.get("Key").and_then(|v| v.as_str()) else {
            continue;
        };
        let key = key.to_lowercase();
        let Some(obj) = entry.as_object() else {
            continue;
        };
        for (lang, value) in obj {
            if lang == "id" || lang == "Key" {
                continue;
            }
            let Some(text) = value.as_str() else {
                continue;
            };
            // 保留所有行（前端支持 white-space: pre-line 换行显示），去掉 D2R 颜色代码 (ÿcX)
            let cleaned = crate::resource::resolver::strip_color_codes(text)
                .trim()
                .to_string();
            conn.execute(
                "INSERT OR REPLACE INTO localized_string (
                    profile_id, namespace, string_key, language, text_value, source_path, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
                params![profile_id, file.role, key, lang, cleaned, file.path],
            )?;
            inserted += 1;
        }
    }
    Ok(inserted)
}

impl Database {
    /// Get a reference to the underlying SQLite connection.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Initialize the database at the default app data path
    pub fn init() -> Result<Self> {
        let db_path = Self::get_db_path();
        log::info!("Initializing database at: {:?}", db_path);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=268435456;
             PRAGMA busy_timeout=5000;"
        ).ok();
        let db = Self { conn };
        db.create_tables()?;
        db.seed_initial_user()?;
        Ok(db)
    }

    /// Open database at a specific path (used for testing)
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.create_tables()?;
        db.seed_initial_user()?;
        Ok(db)
    }

    /// Initialize from an existing connection (used for in-memory testing).
    /// Consumes the connection.
    pub fn init_from_connection(conn: Connection) -> Result<Self> {
        let db = Self { conn };
        db.create_tables()?;
        db.seed_initial_user()?;
        Ok(db)
    }

    /// Consume the Database and return the underlying connection.
    pub fn into_connection(self) -> Connection {
        self.conn
    }

    /// Get a reference to the underlying connection (for query access).
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    /// Get domain-specific repositories from this connection.
    pub fn repos(&self) -> crate::database::repository::Repositories<'_> {
        crate::database::repository::DatabaseReposExt::repos(self.get_connection())
    }
    /// Get the database file path in the user's app data directory
    pub fn get_db_path_clone() -> PathBuf {
        Self::get_db_path()
    }

    fn get_db_path() -> PathBuf {
        let base = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        base.join("database").join("d2r_marketplace.db")
    }

    fn create_tables(&self) -> Result<()> {
        let conn = &self.conn;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                username TEXT DEFAULT 'Jogador',
                token_balance INTEGER DEFAULT 10000
            );

            CREATE TABLE IF NOT EXISTS virtual_items (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                item_code TEXT,
                item_kind TEXT,
                item_type TEXT,
                quality TEXT,
                level INTEGER,
                attributes TEXT,
                source TEXT,
                exported_from TEXT,
                purchased_at TIMESTAMP,
                token_price INTEGER DEFAULT 0,
                status TEXT DEFAULT 'available',
                quantity INTEGER DEFAULT 1,
                unit_price INTEGER DEFAULT 0,
                listed_at TIMESTAMP,
                sell_after_seconds INTEGER,
                profile_id INTEGER DEFAULT 0,
                profile_key TEXT NOT NULL DEFAULT '',
                game_version TEXT DEFAULT '',
                mod_name TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT,
                item_id TEXT,
                token_amount INTEGER,
                description TEXT,
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (item_id) REFERENCES virtual_items(id)
            );

            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS warehouse_items (
                id TEXT PRIMARY KEY,
                item_code TEXT NOT NULL,
                item_name TEXT NOT NULL,
                item_kind TEXT DEFAULT 'misc',
                quality TEXT,
                simple_item INTEGER DEFAULT 0,
                quantity INTEGER DEFAULT 1,
                profile_key TEXT NOT NULL DEFAULT '',
                game_version TEXT DEFAULT '',
                mod_name TEXT DEFAULT '',
                raw_item_bits BLOB,
                raw_bit_length INTEGER DEFAULT 0,
                item_json TEXT DEFAULT '{}',
                stash_name TEXT,
                imported_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                page_name TEXT DEFAULT '默认收藏页',
                tags TEXT DEFAULT '',
                notes TEXT DEFAULT '',
                source_character TEXT,
                source_save_path TEXT,
                slot_equipped TEXT,
                page_index INTEGER DEFAULT 0,
                position_x INTEGER DEFAULT 0,
                position_y INTEGER DEFAULT 0,
                inv_width INTEGER DEFAULT 1,
                inv_height INTEGER DEFAULT 1,
                override_default_page TEXT DEFAULT NULL
            );

            CREATE TABLE IF NOT EXISTS warehouse_default_pages (
                item_code TEXT PRIMARY KEY,
                page_name TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_warehouse_code ON warehouse_items(item_code);
            CREATE INDEX IF NOT EXISTS idx_warehouse_kind ON warehouse_items(item_kind);
            CREATE INDEX IF NOT EXISTS idx_warehouse_page ON warehouse_items(page_name);
            CREATE INDEX IF NOT EXISTS idx_warehouse_profile_key ON warehouse_items(profile_key);
            CREATE INDEX IF NOT EXISTS idx_virtual_items_profile_key ON virtual_items(profile_key);

            CREATE TABLE IF NOT EXISTS resource_profile (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_key TEXT NOT NULL UNIQUE,
                source_kind TEXT NOT NULL,
                mod_name TEXT NOT NULL DEFAULT '',
                game_version TEXT NOT NULL DEFAULT '',
                active_language TEXT NOT NULL DEFAULT 'enUS',
                game_root TEXT NOT NULL DEFAULT '',
                excel_path TEXT NOT NULL DEFAULT '',
                strings_path TEXT NOT NULL DEFAULT '',
                strings_legacy_path TEXT NOT NULL DEFAULT '',
                vanilla_profile_id INTEGER DEFAULT NULL,
                checksum TEXT NOT NULL DEFAULT '',
                source_path TEXT NOT NULL DEFAULT '',
                import_status TEXT NOT NULL DEFAULT '',
                imported_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS resource_file (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                file_type TEXT NOT NULL,
                relation_note TEXT NOT NULL DEFAULT '',
                path TEXT NOT NULL,
                exists_flag INTEGER NOT NULL DEFAULT 0,
                languages_json TEXT NOT NULL DEFAULT '[]',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, role, path),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS localized_string (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                namespace TEXT NOT NULL,
                string_key TEXT NOT NULL,
                language TEXT NOT NULL,
                text_value TEXT NOT NULL,
                source_path TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, namespace, string_key, language, source_path),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_resource_profile_key ON resource_profile(profile_key);
            CREATE INDEX IF NOT EXISTS idx_resource_file_profile ON resource_file(profile_id);
            CREATE INDEX IF NOT EXISTS idx_localized_profile_ns_lang ON localized_string(profile_id, namespace, language);
            CREATE INDEX IF NOT EXISTS idx_localized_profile_key ON localized_string(profile_id, string_key);

            -- === Phase 2: TXT structured import tables ===

            CREATE TABLE IF NOT EXISTS item_base (
                code TEXT NOT NULL,
                profile_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                item_type TEXT NOT NULL DEFAULT '',
                item_category TEXT NOT NULL DEFAULT '',
                inv_width INTEGER DEFAULT 1,
                inv_height INTEGER DEFAULT 1,
                stackable INTEGER DEFAULT 0,
                has_inventory INTEGER DEFAULT 0,
                level INTEGER DEFAULT 0,
                level_req INTEGER DEFAULT 0,
                gem_apply TEXT DEFAULT '',
                is_armor INTEGER DEFAULT 0,
                is_weapon INTEGER DEFAULT 0,
                is_shield INTEGER DEFAULT 0,
                is_mod_item INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (profile_id, code),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS unique_item_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                unique_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                base_code TEXT NOT NULL DEFAULT '',
                level INTEGER DEFAULT 0,
                level_req INTEGER DEFAULT 0,
                is_mod_item INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, unique_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS set_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                set_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, set_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS set_item_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                set_id INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                base_code TEXT NOT NULL DEFAULT '',
                level INTEGER DEFAULT 0,
                level_req INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, item_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE,
                FOREIGN KEY (profile_id, set_id) REFERENCES set_def(profile_id, set_id)
            );

            CREATE TABLE IF NOT EXISTS set_bonus_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                set_id INTEGER NOT NULL,
                piece_count INTEGER NOT NULL,   -- 2..5 = 多件套; 6 = 全套 (FCode)
                stat_id INTEGER NOT NULL,
                param INTEGER DEFAULT 0,
                min_value INTEGER DEFAULT 0,
                max_value INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, set_id, piece_count, stat_id, param),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE,
                FOREIGN KEY (profile_id, set_id) REFERENCES set_def(profile_id, set_id)
            );

            CREATE TABLE IF NOT EXISTS resource_import_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                table_name TEXT NOT NULL,
                rows_count INTEGER DEFAULT 0,
                source TEXT NOT NULL DEFAULT 'txt',
                started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                completed_at TIMESTAMP,
                status TEXT DEFAULT 'pending',
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            -- === Phase 2.2: Runeword / Stat / Skill definitions ===

            CREATE TABLE IF NOT EXISTS runeword_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                runeword_key TEXT NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                rune_codes TEXT NOT NULL DEFAULT '',
                allowed_base_types TEXT NOT NULL DEFAULT '',
                sockets INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, runeword_key),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS stat_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                stat_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                save_bits INTEGER DEFAULT 0,
                save_param_bits INTEGER DEFAULT 0,
                save_add INTEGER DEFAULT 0,
                signed INTEGER DEFAULT 0,
                encoding INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, stat_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS skill_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                skill_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                char_class TEXT NOT NULL DEFAULT '',
                skill_tab INTEGER DEFAULT 0,
                is_aura INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, skill_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS skill_tab_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                tab_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                char_class TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, tab_id),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            -- === Phase 2.2: MagicPrefix / MagicSuffix affix definitions ===

            CREATE TABLE IF NOT EXISTS item_affix_def (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                affix_id INTEGER NOT NULL,
                name_en TEXT NOT NULL DEFAULT '',
                affix_type TEXT NOT NULL DEFAULT '',
                group_id INTEGER NOT NULL DEFAULT 0,
                level INTEGER NOT NULL DEFAULT 0,
                level_req INTEGER NOT NULL DEFAULT 0,
                exclude_level INTEGER NOT NULL DEFAULT 0,
                exclude_counter INTEGER NOT NULL DEFAULT 0,
                mod1code TEXT NOT NULL DEFAULT '',
                mod1param INTEGER NOT NULL DEFAULT 0,
                mod1min INTEGER NOT NULL DEFAULT 0,
                mod1max INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_affix_profile_type ON item_affix_def(profile_id, affix_type);

            -- US-022: Add index on localized_string for fast warmup queries
            CREATE INDEX IF NOT EXISTS idx_localized_lookup
                ON localized_string(profile_id, namespace, string_key, language);
            
            -- US-024: Also index by profile_id+namespace for the warmup SELECT
            CREATE INDEX IF NOT EXISTS idx_localized_warmup
                ON localized_string(profile_id);

            -- === Grail tracking ===

            CREATE TABLE IF NOT EXISTS grail_tracking (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                item_key TEXT NOT NULL,
                item_type TEXT NOT NULL,
                item_code TEXT NOT NULL DEFAULT '',
                name_en TEXT NOT NULL DEFAULT '',
                found INTEGER NOT NULL DEFAULT 0,
                found_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(profile_id, item_key),
                FOREIGN KEY (profile_id) REFERENCES resource_profile(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_grail_profile ON grail_tracking(profile_id, item_type);

            CREATE INDEX IF NOT EXISTS idx_item_base_type ON item_base(profile_id, item_type);
            CREATE INDEX IF NOT EXISTS idx_unique_profile ON unique_item_def(profile_id);
            CREATE INDEX IF NOT EXISTS idx_set_item_profile ON set_item_def(profile_id);
            CREATE INDEX IF NOT EXISTS idx_import_log_profile ON resource_import_log(profile_id);

            -- Phase 2.2 indices
            CREATE INDEX IF NOT EXISTS idx_runeword_profile ON runeword_def(profile_id);
            CREATE INDEX IF NOT EXISTS idx_stat_profile ON stat_def(profile_id);
            CREATE INDEX IF NOT EXISTS idx_skill_profile ON skill_def(profile_id);
            CREATE INDEX IF NOT EXISTS idx_skilltab_profile ON skill_tab_def(profile_id);
            ",
        )?;
        // 兼容旧库：resource_profile 初版没有 game_root 字段。
        let _ = conn.execute(
            "ALTER TABLE resource_profile ADD COLUMN game_root TEXT NOT NULL DEFAULT ''",
            [],
        );
        // 兼容旧库：warehouse_items 初版没有 profile_key 字段。
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN profile_key TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "UPDATE warehouse_items
             SET profile_key = CASE
                 WHEN COALESCE(mod_name, '') = '' AND COALESCE(game_version, '') = '' THEN 'vanilla:default'
                 WHEN COALESCE(mod_name, '') = '' THEN 'vanilla:' || game_version
                 WHEN COALESCE(game_version, '') = '' THEN 'mod:' || mod_name
                 ELSE 'mod:' || mod_name || ':' || game_version
             END
             WHERE COALESCE(profile_key, '') = ''",
            [],
        );
        // 兼容旧库：warehouse_items 初版没有 source_character / slot_equipped 字段。
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN source_character TEXT DEFAULT NULL",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN source_save_path TEXT DEFAULT NULL",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN slot_equipped TEXT DEFAULT NULL",
            [],
        );
        // 兼容旧库：warehouse_items 缺少 page_index / position / inv 字段。
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN page_index INTEGER DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN position_x INTEGER DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN position_y INTEGER DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN inv_width INTEGER DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN inv_height INTEGER DEFAULT 1",
            [],
        );
        // 兼容旧库：warehouse_items 缺少 override_default_page 字段。
        let _ = conn.execute(
            "ALTER TABLE warehouse_items ADD COLUMN override_default_page TEXT DEFAULT NULL",
            [],
        );
        // 兼容旧库：virtual_items 初版没有资源画像隔离字段。
        let _ = conn.execute(
            "ALTER TABLE virtual_items ADD COLUMN profile_id INTEGER DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE virtual_items ADD COLUMN profile_key TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE virtual_items ADD COLUMN game_version TEXT DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE virtual_items ADD COLUMN mod_name TEXT DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "UPDATE virtual_items
             SET profile_key = CASE
                 WHEN COALESCE(mod_name, '') = '' AND COALESCE(game_version, '') = '' THEN 'vanilla:default'
                 WHEN COALESCE(mod_name, '') = '' THEN 'vanilla:' || game_version
                 WHEN COALESCE(game_version, '') = '' THEN 'mod:' || mod_name
                 ELSE 'mod:' || mod_name || ':' || game_version
             END
             WHERE COALESCE(profile_key, '') = ''",
            [],
        );
        Ok(())
    }

    fn seed_initial_user(&self) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO user (id, username, token_balance) VALUES (1, 'Jogador', 10000)",
            [],
        )?;
        Ok(())
    }

    // ── User / Balance ──────────────────────────────────────────

    pub fn get_token_balance(&self) -> Result<i64> {
        let result = self
            .conn
            .query_row("SELECT token_balance FROM user WHERE id = 1", [], |row| {
                row.get(0)
            })?;
        Ok(result)
    }

    pub fn update_token_balance(&self, amount: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE user SET token_balance = token_balance + ? WHERE id = 1",
            params![amount],
        )?;
        Ok(())
    }

    // ── Virtual Items ──────────────────────────────────────────

    pub fn add_virtual_item(&self, item: &VirtualItem) -> Result<()> {
        self.conn.execute(
            "INSERT INTO virtual_items (
                id, name, item_code, item_kind, item_type, quality, level,
                attributes, source, exported_from, purchased_at, token_price,
                status, quantity, unit_price, listed_at, sell_after_seconds,
                profile_id, profile_key, game_version, mod_name
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                item.id,
                item.name,
                item.item_code,
                item.item_kind,
                item.item_type,
                item.quality,
                item.level,
                item.attributes,
                item.source,
                item.exported_from,
                item.purchased_at,
                item.token_price,
                item.status,
                item.quantity,
                item.unit_price,
                item.listed_at,
                item.sell_after_seconds,
                item.profile_id,
                item.profile_key,
                item.game_version,
                item.mod_name,
            ],
        )?;
        Ok(())
    }

    pub fn get_listed_items(&self) -> Result<Vec<ListedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from
             FROM virtual_items
             WHERE status = 'listed'
             ORDER BY listed_at ASC",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(ListedItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                    unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                    listed_at: row.get(4)?,
                    sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    status: row.get(6)?,
                    item_code: row.get(7)?,
                    item_kind: row.get(8)?,
                    quality: row.get(9)?,
                    listed_by: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn get_listed_items_in_profile(&self, profile_key: &str) -> Result<Vec<ListedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from
             FROM virtual_items
             WHERE status = 'listed' AND profile_key = ?
             ORDER BY listed_at ASC",
        )?;

        let items = stmt
            .query_map(params![profile_key], |row| {
                Ok(ListedItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                    unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                    listed_at: row.get(4)?,
                    sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    status: row.get(6)?,
                    item_code: row.get(7)?,
                    item_kind: row.get(8)?,
                    quality: row.get(9)?,
                    listed_by: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn get_listed_items_paginated(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let offset_clause = offset.map(|o| format!(" OFFSET {}", o)).unwrap_or_default();
        let sql = format!(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from AS listed_by
             FROM virtual_items
             WHERE status = 'listed'
             ORDER BY listed_at DESC{}{}",
            limit_clause, offset_clause
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let items = stmt.query_map([], |row| {
            Ok(ListedItem {
                id: row.get(0)?,
                name: row.get(1)?,
                quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                listed_at: row.get(4)?,
                sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                status: row.get(6)?,
                item_code: row.get(7)?,
                item_kind: row.get(8)?,
                quality: row.get(9)?,
                listed_by: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_listed_items_in_profile_paginated(&self, profile_key: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let offset_clause = offset.map(|o| format!(" OFFSET {}", o)).unwrap_or_default();
        let sql = format!(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from AS listed_by
             FROM virtual_items
             WHERE status = 'listed' AND profile_key = ?
             ORDER BY listed_at ASC{}{}",
            limit_clause, offset_clause
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let items = stmt
            .query_map(params![profile_key], |row| {
                Ok(ListedItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                    unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                    listed_at: row.get(4)?,
                    sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    status: row.get(6)?,
                    item_code: row.get(7)?,
                    item_kind: row.get(8)?,
                    quality: row.get(9)?,
                    listed_by: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_listed_item_by_id(&self, item_id: &str) -> Result<Option<ListedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from
             FROM virtual_items WHERE id = ?",
        )?;

        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(ListedItem {
                id: row.get(0)?,
                name: row.get(1)?,
                quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                listed_at: row.get(4)?,
                sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                status: row.get(6)?,
                item_code: row.get(7)?,
                item_kind: row.get(8)?,
                quality: row.get(9)?,
                listed_by: row.get(10)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_listed_item_by_id_in_profile(&self, item_id: &str, profile_key: &str) -> Result<Option<ListedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds, status,
                    item_code, item_kind, quality, exported_from
             FROM virtual_items
             WHERE id = ? AND profile_key = ?",
        )?;

        let mut rows = stmt.query(params![item_id, profile_key])?;
        match rows.next()? {
            Some(row) => Ok(Some(ListedItem {
                id: row.get(0)?,
                name: row.get(1)?,
                quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                listed_at: row.get(4)?,
                sell_after_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                status: row.get(6)?,
                item_code: row.get(7)?,
                item_kind: row.get(8)?,
                quality: row.get(9)?,
                listed_by: row.get(10)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_virtual_items(&self, status: &str) -> Result<Vec<VirtualItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM virtual_items WHERE status = ?",
        )?;

        let items = stmt
            .query_map(params![status], |row| {
                Ok(VirtualItem {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    item_code: row.get("item_code")?,
                    item_kind: row.get("item_kind")?,
                    item_type: row.get("item_type")?,
                    quality: row.get("quality")?,
                    level: row.get("level")?,
                    attributes: row.get("attributes")?,
                    source: row.get("source")?,
                    exported_from: row.get("exported_from")?,
                    purchased_at: row.get("purchased_at")?,
                    token_price: row.get("token_price")?,
                    status: row.get("status")?,
                    quantity: row.get("quantity")?,
                    unit_price: row.get("unit_price")?,
                    listed_at: row.get("listed_at")?,
                    sell_after_seconds: row.get("sell_after_seconds")?,
                    profile_id: row.get("profile_id")?,
                    profile_key: row.get("profile_key")?,
                    game_version: row.get("game_version")?,
                    mod_name: row.get("mod_name")?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn get_virtual_items_in_profile(&self, status: &str, profile_key: &str) -> Result<Vec<VirtualItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM virtual_items WHERE status = ? AND profile_key = ?",
        )?;

        let items = stmt
            .query_map(params![status, profile_key], |row| {
                Ok(VirtualItem {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    item_code: row.get("item_code")?,
                    item_kind: row.get("item_kind")?,
                    item_type: row.get("item_type")?,
                    quality: row.get("quality")?,
                    level: row.get("level")?,
                    attributes: row.get("attributes")?,
                    source: row.get("source")?,
                    exported_from: row.get("exported_from")?,
                    purchased_at: row.get("purchased_at")?,
                    token_price: row.get("token_price")?,
                    status: row.get("status")?,
                    quantity: row.get("quantity")?,
                    unit_price: row.get("unit_price")?,
                    listed_at: row.get("listed_at")?,
                    sell_after_seconds: row.get("sell_after_seconds")?,
                    profile_id: row.get("profile_id")?,
                    profile_key: row.get("profile_key")?,
                    game_version: row.get("game_version")?,
                    mod_name: row.get("mod_name")?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn get_virtual_item_by_id(&self, item_id: &str) -> Result<Option<VirtualItem>> {
        let mut stmt = self.conn.prepare("SELECT * FROM virtual_items WHERE id = ?")?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(VirtualItem {
                id: row.get("id")?,
                name: row.get("name")?,
                item_code: row.get("item_code")?,
                item_kind: row.get("item_kind")?,
                item_type: row.get("item_type")?,
                quality: row.get("quality")?,
                level: row.get("level")?,
                attributes: row.get("attributes")?,
                source: row.get("source")?,
                exported_from: row.get("exported_from")?,
                purchased_at: row.get("purchased_at")?,
                token_price: row.get("token_price")?,
                status: row.get("status")?,
                quantity: row.get("quantity")?,
                unit_price: row.get("unit_price")?,
                listed_at: row.get("listed_at")?,
                sell_after_seconds: row.get("sell_after_seconds")?,
                profile_id: row.get("profile_id")?,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
            })),
            None => Ok(None),
        }
    }

    pub fn mark_listing_cancelled(&self, item_id: &str, profile_key: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE virtual_items SET status = 'cancelled' WHERE id = ? AND status = 'listed' AND profile_key = ?",
            params![item_id, profile_key],
        )?;
        Ok(affected > 0)
    }

    /// 更新在售物品的单价(改价功能)。
    /// 仅 status='listed' 的物品可改价, 返回 true 表示改价成功, false 表示不存在或已下架。
    pub fn update_listing_price(&self, item_id: &str, new_unit_price: i64, profile_key: &str) -> Result<bool> {
        if new_unit_price < 1 {
            return Ok(false);
        }
        let affected = self.conn.execute(
            "UPDATE virtual_items
             SET unit_price = ?, token_price = ?
             WHERE id = ? AND status = 'listed' AND profile_key = ?",
            params![new_unit_price, new_unit_price, item_id, profile_key],
        )?;
        Ok(affected > 0)
    }

    pub fn mark_listing_sold(&self, item_id: &str) -> Result<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE virtual_items SET status = 'sold', purchased_at = ? WHERE id = ? AND status = 'listed'",
            params![now, item_id],
        )?;
        Ok(affected > 0)
    }

    pub fn mark_item_as_sold(&self, item_id: &str, profile_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE virtual_items SET status = 'sold' WHERE id = ? AND profile_key = ?",
            params![item_id, profile_key],
        )?;
        Ok(())
    }

    pub fn mark_item_as_imported(&self, item_id: &str, profile_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE virtual_items SET status = 'imported' WHERE id = ? AND profile_key = ?",
            params![item_id, profile_key],
        )?;
        Ok(())
    }

    // ── Transactions ───────────────────────────────────────────

    pub fn add_transaction(
        &self,
        tx_type: &str,
        item_id: Option<&str>,
        token_amount: i64,
        description: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO transactions (type, item_id, token_amount, description) VALUES (?, ?, ?, ?)",
            params![tx_type, item_id, token_amount, description],
        )?;
        Ok(())
    }

    /// 读取交易历史(默认按时间倒序,最新在前)。
    /// - `limit`: 最多返回 N 条(默认 200)
    /// - `tx_type`: 可选, 只返回指定类型(如 "buy_import" / "list" / "sell" / "export")
    pub fn get_transactions(&self, limit: i64, tx_type: Option<&str>) -> Result<Vec<Transaction>> {
        let limit = if limit <= 0 { 200 } else { limit.min(1000) };
        let (sql, use_filter): (String, bool) = if tx_type.is_some() {
            (
                "SELECT id, type, item_id, token_amount, description, date
                 FROM transactions
                 WHERE type = ?
                 ORDER BY id DESC
                 LIMIT ?"
                    .to_string(),
                true,
            )
        } else {
            (
                "SELECT id, type, item_id, token_amount, description, date
                 FROM transactions
                 ORDER BY id DESC
                 LIMIT ?"
                    .to_string(),
                false,
            )
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let mapper = |row: &rusqlite::Row| -> rusqlite::Result<Transaction> {
            Ok(Transaction {
                id: row.get(0)?,
                tx_type: row.get(1)?,
                item_id: row.get(2)?,
                token_amount: row.get(3)?,
                description: row.get(4)?,
                date: row.get(5)?,
            })
        };
        let items = if use_filter {
            stmt.query_map(params![tx_type.ok_or_else(|| rusqlite::Error::InvalidQuery)?, limit], mapper)?
                .collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map(params![limit], mapper)?
                .collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    // ── Process due listings (auto-sell) ───────────────────────

    pub fn process_due_listings(&self) -> Result<Vec<SoldItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds
             FROM virtual_items WHERE status = 'listed'",
        )?;

        let due_items: Vec<SoldItem> = stmt
            .query_map([], |row| {
                let listed_at_str: String = row.get(4)?;
                let sell_after: i64 = row.get::<_, Option<i64>>(5)?.unwrap_or(0);
                Ok(SoldItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quantity: row.get::<_, Option<i64>>(2)?.unwrap_or(1) as i32,
                    unit_price: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as i32,
                    listed_at: listed_at_str,
                    sell_after_seconds: sell_after,
                })
            })?
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(|item| {
                let listed_dt = chrono::DateTime::parse_from_rfc3339(&item.listed_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok();
                if let Some(dt) = listed_dt {
                    let elapsed = (chrono::Utc::now() - dt).num_seconds();
                    elapsed >= item.sell_after_seconds
                } else {
                    false
                }
            })
            .collect();

        let mut sold = Vec::new();
        for item in &due_items {
            let total_tokens = (item.quantity as i64) * (item.unit_price as i64);
            let now = chrono::Utc::now().to_rfc3339();

            self.conn.execute(
                "UPDATE virtual_items SET status = 'sold', purchased_at = ? WHERE id = ? AND status = 'listed'",
                params![now, item.id],
            )?;

            if self.conn.changes() > 0 {
                self.conn.execute(
                    "UPDATE user SET token_balance = token_balance + ? WHERE id = 1",
                    params![total_tokens],
                )?;

                self.conn.execute(
                    "INSERT INTO transactions (type, item_id, token_amount, description, date) VALUES (?, ?, ?, ?, ?)",
                    params![
                        "sale_credit",
                        item.id,
                        total_tokens,
                        format!("Auto-sale of {} x{}", item.name, item.quantity),
                        now,
                    ],
                )?;

                sold.push(SoldItem {
                    id: item.id.clone(),
                    name: item.name.clone(),
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    listed_at: item.listed_at.clone(),
                    sell_after_seconds: item.sell_after_seconds,
                });
            }
        }

        Ok(sold)
    }

    // ── App Config ─────────────────────────────────────────────

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM app_config WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn upsert_resource_manifest(&self, manifest: &ResourceManifest) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO resource_profile (
                profile_key, source_kind, mod_name, game_version, active_language,
                game_root, excel_path, strings_path, strings_legacy_path,
                vanilla_profile_id, checksum, source_path, imported_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(profile_key) DO UPDATE SET
                source_kind=excluded.source_kind,
                mod_name=excluded.mod_name,
                game_version=excluded.game_version,
                active_language=excluded.active_language,
                game_root=excluded.game_root,
                excel_path=excluded.excel_path,
                strings_path=excluded.strings_path,
                strings_legacy_path=excluded.strings_legacy_path,
                checksum=excluded.checksum,
                source_path=excluded.source_path,
                updated_at=CURRENT_TIMESTAMP",
            params![
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
        )?;
        let profile_id: i64 = self.conn.query_row(
            "SELECT id FROM resource_profile WHERE profile_key = ?1",
            params![manifest.profile_id],
            |row| row.get(0),
        )?;

        self.conn.execute("DELETE FROM resource_file WHERE profile_id = ?1", params![profile_id])?;
        for file in manifest.txt_files.iter().chain(manifest.json_files.iter()) {
            let languages_json = serde_json::to_string(&file.languages).unwrap_or_else(|_| "[]".to_string());
            self.conn.execute(
                "INSERT INTO resource_file (
                    profile_id, role, file_type, relation_note, path, exists_flag, languages_json, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
                params![
                    profile_id,
                    file.role,
                    file.file_type,
                    file.relation,
                    file.path,
                    if file.exists { 1 } else { 0 },
                    languages_json,
                ],
            )?;
        }
        Ok(profile_id)
    }

    pub fn import_localized_strings_from_manifest(&self, manifest: &ResourceManifest) -> Result<usize> {
        let profile_id = self.upsert_resource_manifest(manifest)?;
        let existing_count: i64 = self.conn.query_row(
            "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
            params![profile_id],
            |row| row.get(0),
        )?;
        if existing_count > 0 {
            return Ok(existing_count as usize);
        }

        let mut inserted = 0usize;
        for file in &manifest.json_files {
            inserted += import_string_file(&self.conn, profile_id, file)?;
        }
        Ok(inserted)
    }

    /// Check if game definition data already exists for a profile.
    /// Returns true if item_base has at least one row.
    pub fn has_game_definitions(&self, profile_id: i64) -> bool {
        
        self
            .conn
            .query_row(
                "SELECT COUNT(1) FROM item_base WHERE profile_id = ?1",
                params![profile_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0
    }

    pub fn has_localized_strings(&self, profile_id: i64) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(1) FROM localized_string WHERE profile_id = ?1",
                params![profile_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0
    }

    /// Import game definition tables (item_base, unique_item_def, set_def, set_item_def)
    /// from TXT files or hardcoded constants.
    /// Returns per-table import results.
    pub fn import_game_definitions(&self, manifest: &ResourceManifest) -> Result<Vec<crate::resource::importer::ImportResult>> {
        let profile_id = self.upsert_resource_manifest(manifest)?;
        let importer = ResourceImporter::new(&self.conn, profile_id);
        Ok(importer.import_all(&manifest.excel_path))
    }

    pub fn count_localized_strings_for_profile(&self, profile_key: &str) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(1)
             FROM localized_string ls
             JOIN resource_profile rp ON rp.id = ls.profile_id
             WHERE rp.profile_key = ?1",
            params![profile_key],
            |row| row.get(0),
        )
    }

    // ── Warehouse (Extended Stash) ──────────────────────────────

    /// Add an item to the warehouse
    pub fn warehouse_add(&self, item: &WarehousedItem) -> Result<()> {
        self.conn.execute(
            "INSERT INTO warehouse_items (id, item_code, item_name, item_kind, quality,
                simple_item, quantity, profile_key, game_version, mod_name, raw_item_bits, raw_bit_length,
                item_json, stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped,
                page_index, position_x, position_y, inv_width, inv_height)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
            params![
                item.id, item.item_code, item.item_name, item.item_kind, item.quality,
                item.simple_item as i32, item.quantity, item.profile_key, item.game_version, item.mod_name,
                item.raw_item_bits, item.raw_bit_length as i64, item.item_json,
                item.stash_name, item.imported_at, item.page_name, item.tags, item.notes,
                item.source_character, item.source_save_path, item.slot_equipped,
                item.page_index, item.position_x, item.position_y,
                item.inv_width, item.inv_height,
            ],
        )?;
        Ok(())
    }

    /// Get all warehoused items
    pub fn warehouse_list_all(&self) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped,
                inv_width, inv_height
             FROM warehouse_items ORDER BY imported_at DESC"
        )?;
        let items = stmt.query_map([], |row| {
            Ok(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn warehouse_list_by_context(&self, mod_name: &str, game_version: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped,
                inv_width, inv_height
             FROM warehouse_items
             WHERE mod_name = ?1 AND game_version = ?2
             ORDER BY imported_at DESC"
        )?;
        let items = stmt.query_map(params![mod_name, game_version], |row| {
            Ok(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn warehouse_list_by_profile(&self, profile_key: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
             FROM warehouse_items
             WHERE profile_key = ?1
             ORDER BY imported_at DESC"
        )?;
        let items = stmt.query_map(params![profile_key], |row| {
            Ok(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    /// Get warehoused items by page/collection name
    pub fn warehouse_list_by_page(&self, page_name: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
             FROM warehouse_items WHERE page_name = ?1 ORDER BY item_kind, item_name"
        )?;
        let items = stmt.query_map(params![page_name], |row| {
            Ok(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn warehouse_list_by_page_in_profile(&self, profile_key: &str, page_name: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
             FROM warehouse_items
             WHERE profile_key = ?1 AND page_name = ?2
             ORDER BY item_kind, item_name"
        )?;
        let items = stmt.query_map(params![profile_key, page_name], |row| {
            Ok(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    /// Get a single warehoused item by ID
    pub fn warehouse_get(&self, item_id: &str) -> Result<Option<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
             FROM warehouse_items WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })),
            None => Ok(None),
        }
    }

    pub fn warehouse_get_in_profile(&self, profile_key: &str, item_id: &str) -> Result<Option<WarehousedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_code, item_name, item_kind, quality, simple_item, quantity,
                profile_key, game_version, mod_name, raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
             FROM warehouse_items
             WHERE profile_key = ?1 AND id = ?2"
        )?;
        let mut rows = stmt.query(params![profile_key, item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(WarehousedItem {
                id: row.get("id")?,
                item_code: row.get("item_code")?,
                item_name: row.get("item_name")?,
                item_kind: row.get("item_kind")?,
                quality: row.get("quality")?,
                simple_item: row.get::<_, i32>("simple_item")? != 0,
                quantity: row.get::<_, i64>("quantity")? as u32,
                profile_key: row.get("profile_key")?,
                game_version: row.get("game_version")?,
                mod_name: row.get("mod_name")?,
                raw_item_bits: row.get("raw_item_bits")?,
                raw_bit_length: row.get::<_, i64>("raw_bit_length")? as usize,
                item_json: row.get("item_json")?,
                stash_name: row.get("stash_name")?,
                imported_at: row.get("imported_at")?,
                page_name: row.get("page_name")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                source_character: row.get("source_character")?,
                source_save_path: row.get("source_save_path")?,
                slot_equipped: row.get("slot_equipped")?,
                page_index: row.get("page_index").unwrap_or(0),
                position_x: row.get("position_x").unwrap_or(0),
                position_y: row.get("position_y").unwrap_or(0),
                inv_width: row.get("inv_width").unwrap_or(1),
                inv_height: row.get("inv_height").unwrap_or(1),
            })),
            None => Ok(None),
        }
    }

    /// Remove an item from the warehouse
    pub fn warehouse_remove(&self, item_id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM warehouse_items WHERE id = ?1",
            params![item_id],
        )?;
        Ok(affected > 0)
    }

    /// Partial-withdraw side effect: 在仓库行上把 quantity 减少 + 重编码 raw_item_bits。
    /// 用于 stash 取回 N/M 时,DB 行保留但 amount 改为 (M-N),bitstream 同步重写。
    /// 非 partial 路径不要调这个,直接走 warehouse_remove_in_profile。
    pub fn warehouse_partial_withdraw(
        &self,
        item_id: &str,
        new_quantity: u32,
        new_raw_bits: &[u8],
    ) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE warehouse_items
             SET quantity = ?1, raw_item_bits = ?2
             WHERE id = ?3",
            params![new_quantity as i64, new_raw_bits, item_id],
        )?;
        Ok(affected > 0)
    }

    /// Re-derive inv_width / inv_height for every warehouse row from item_code,
    /// using the legacy get_item_inventory_size resolver. Used to backfill rows
    /// that were inserted before the columns were populated by warehouse_add.
    /// Returns the number of rows actually updated.
    pub fn warehouse_backfill_dims(&self) -> Result<usize> {
        use crate::protocol::d2i::legacy::item::get_item_inventory_size;
        let mut stmt = self.conn.prepare("SELECT id, item_code FROM warehouse_items")?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get::<_, String>("id")?, row.get::<_, String>("item_code")?)))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        let mut updated = 0usize;
        for (id, code) in rows {
            let (w, h) = get_item_inventory_size(&code);
            let n = self.conn.execute(
                "UPDATE warehouse_items SET inv_width = ?1, inv_height = ?2 WHERE id = ?3",
                params![w as i32, h as i32, id],
            )?;
            if n > 0 { updated += n; }
        }
        Ok(updated)
    }

    pub fn warehouse_remove_in_profile(&self, profile_key: &str, item_id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM warehouse_items WHERE profile_key = ?1 AND id = ?2",
            params![profile_key, item_id],
        )?;
        Ok(affected > 0)
    }

    /// Get distinct page/collection names
    pub fn warehouse_list_pages(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT page_name FROM warehouse_items ORDER BY page_name"
        )?;
        let pages = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;
        Ok(pages)
    }

    pub fn warehouse_list_pages_in_profile(&self, profile_key: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT page_name FROM warehouse_items WHERE profile_key = ?1 ORDER BY page_name"
        )?;
        let pages = stmt.query_map(params![profile_key], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;
        Ok(pages)
    }

    /// Update warehouse item metadata (page, tags, notes)
    pub fn warehouse_update_meta(&self, item_id: &str, page_name: &str, tags: &str, notes: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE warehouse_items SET page_name = ?1, tags = ?2, notes = ?3 WHERE id = ?4",
            params![page_name, tags, notes, item_id],
        )?;
        Ok(affected > 0)
    }

    pub fn warehouse_update_meta_in_profile(&self, profile_key: &str, item_id: &str, page_name: &str, tags: &str, notes: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE warehouse_items
             SET page_name = ?1, tags = ?2, notes = ?3
             WHERE profile_key = ?4 AND id = ?5",
            params![page_name, tags, notes, profile_key, item_id],
        )?;
        Ok(affected > 0)
    }

    // ── Default page (per-code + per-item override) ─────────────

    /// Set / upsert per-code default page (all items with this code share it).
    /// Global behavior — not profile-scoped, since the default applies to
    /// deposit-time page routing regardless of which profile is active.
    pub fn warehouse_set_code_default(&self, item_code: &str, page_name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO warehouse_default_pages (item_code, page_name, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(item_code) DO UPDATE SET
                 page_name = excluded.page_name,
                 updated_at = CURRENT_TIMESTAMP",
            params![item_code, page_name],
        )?;
        Ok(())
    }

    /// Clear per-code default page. Returns true iff a row was deleted.
    pub fn warehouse_clear_code_default(&self, item_code: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM warehouse_default_pages WHERE item_code = ?1",
            params![item_code],
        )?;
        Ok(affected > 0)
    }

    /// Get per-code default page, if any.
    pub fn warehouse_get_code_default(&self, item_code: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT page_name FROM warehouse_default_pages WHERE item_code = ?1",
        )?;
        let mut rows = stmt.query(params![item_code])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Set per-item override (this single warehouse row goes to a specific page,
    /// ignoring the per-code default). Returns false if the item doesn't exist.
    pub fn warehouse_set_item_default(&self, item_id: &str, page_name: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE warehouse_items SET override_default_page = ?1 WHERE id = ?2",
            params![page_name, item_id],
        )?;
        Ok(affected > 0)
    }

    /// Clear per-item override (falls back to per-code default on next deposit).
    pub fn warehouse_clear_item_default(&self, item_id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE warehouse_items SET override_default_page = NULL WHERE id = ?1",
            params![item_id],
        )?;
        Ok(affected > 0)
    }

    /// Get per-item override page, if any.
    pub fn warehouse_get_item_default(&self, item_id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT override_default_page FROM warehouse_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(row.get::<_, Option<String>>(0)?),
            None => Ok(None),
        }
    }
}

/// Get the app data directory (cross-platform)
fn dirs_next() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("D2RMarketplace"))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("Library").join("Application Support").join("D2RMarketplace"))
    } else {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("D2RMarketplace"))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|p| PathBuf::from(p).join(".local").join("share").join("D2RMarketplace"))
            })
    }
}
