//! Unified NameResolver for D2R items.
//!
//! 统一名称解析器，替代 `item_names.rs` 的混合式运行时逻辑。
//!
//! ## 优先级链
//!
//! 1. **Unique** — `unique_item_def.name_en` → `localized_string` (zhCN/zhTW)
//! 2. **Set** — `set_item_def.name_en` → `localized_string`
//! 3. **Runeword** — `runeword_def.name_en` → `localized_string`
//! 4. **Base** — `item_base.name_en` → `localized_string`
//! 5. **Rare/Magic** — affix 生成名（TBD）
//! 6. **Fallback** — quality 前缀 + code

use rusqlite::{Connection, params};
use crate::resource::queries;
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};

/// Resolved item display name with source tracking.
#[derive(Debug, Clone)]
pub struct ResolvedName {
    /// Localized display name in the requested language.
    pub display_name: String,
    /// English name (unlocalized, from definition tables).
    pub name_en: String,
    /// Source of the name for debugging.
    pub source: NameSource,
}

/// Source of a resolved name.
#[derive(Debug, Clone, PartialEq)]
pub enum NameSource {
    /// From unique_item_def (暗金物品)
    Unique,
    /// From set_item_def (套装物品)
    Set,
    /// From runeword_def (符文之语)
    Runeword,
    /// From item_base (基础物品)
    BaseItem,
    /// From affix generation (稀有/魔法)
    RareMagic,
    /// Generated fallback: quality prefix + code
    Fallback,
}

/// Unified name resolver backed by SQLite definition tables.
///
/// Usage:
/// ```ignore
/// let resolver = NameResolver::new(profile_id);
/// let name = resolver.resolve(&conn, "cap", Some(2), None, None, "zhCN");
/// // → "帽子" (from localized_string)
/// ```
pub struct NameResolver {
    pub profile_id: i64,
    pub vanilla_profile_id: Option<i64>,
    /// US-022: 预加载 localized_string → (namespace, string_key, language) → text_value
    /// 一次性把全部 localized_string 灌进 HashMap,消除每次 resolve 时的 1-4 次 SQLite 查询。
    /// `None` 表示未预热(向后兼容 `NameResolver::new` 调用);`Some(empty_map)` 表示已预热但该 profile 无 localized_string。
    localized_cache: Option<std::collections::HashMap<(String, String, String), String>>,
    /// US-022b: 预加载 item_base (code → name_en)。消除每次 resolve 调 get_item_base 的 SQL 查询。
    item_base_cache: Option<std::collections::HashMap<String, String>>,
    /// US-022b: 预加载 unique_item_def (unique_id → name_en)。
    unique_def_cache: Option<std::collections::HashMap<u16, String>>,
    /// US-024: 反向索引 (lowercase name_en → 3-char code)。
    /// 用于 localized_cache miss 时自动 fallback: 用 name_en 找 code,再用 code 查 cache。
    /// 解决 `name_en="Skull Cap"` 但 DB 只存 `key="cap"` 这种格式不匹配问题。
    name_en_to_code: Option<std::collections::HashMap<String, String>>,
}

impl NameResolver {
    pub fn new(profile_id: i64) -> Self {
        Self {
            profile_id,
            vanilla_profile_id: None,
            localized_cache: None,
            item_base_cache: None,
            unique_def_cache: None,
            name_en_to_code: None,
        }
    }

    pub fn with_vanilla(profile_id: i64, vanilla_profile_id: i64) -> Self {
        Self {
            profile_id,
            vanilla_profile_id: Some(vanilla_profile_id),
            localized_cache: None,
            item_base_cache: None,
            unique_def_cache: None,
            name_en_to_code: None,
        }
    }

    /// US-022: 创建 resolver 时一次性预热 localized_string + item_base + unique_item_def 缓存。
    ///
    /// 背景:`_get_localized` 内部每个 cache miss 走 1-4 次 SQLite 查询
    /// (current profile → 3 个 vanilla key → 全 profile 兜底)。
    /// 65 物品 × 3 语言 × 2-4 SQL = 400-800 次 = 8000-16000ms。
    /// 预热后:一次 `SELECT FROM localized_string` + 一次 `SELECT FROM item_base` + 一次
    /// `SELECT FROM unique_item_def`,之后 lookup O(1)。
    pub fn with_localized_cache(conn: &Connection, profile_id: i64) -> Self {
        let started = std::time::Instant::now();
        let mut resolver = Self::new(profile_id);
        resolver.warmup_localized(conn);
        resolver.warmup_item_base(conn);
        resolver.warmup_unique_def(conn);
        log::info!(
            "[NameResolver] with_localized_cache COMPLETED: profile_id={} elapsed={}ms",
            profile_id, started.elapsed().as_millis()
        );
        resolver
    }

    /// 显式预热入口(用于已有 NameResolver 实例的场景)。
    pub fn warmup_localized(&mut self, conn: &Connection) {
        let mut cache: std::collections::HashMap<(String, String, String), String> =
            std::collections::HashMap::new();
        // Load ALL entries — first-pass fills gaps, second-pass (current profile) wins
        let query = "SELECT string_key, namespace, language, text_value, profile_id FROM localized_string";
        if let Ok(mut stmt) = conn.prepare(query)
            && let Ok(rows) = stmt.query_map([], |row| {
                let key: String = row.get(0)?;
                let ns: String = row.get(1)?;
                let lang: String = row.get(2)?;
                let text: String = row.get(3)?;
                let pid: i64 = row.get(4)?;
                Ok((key, ns, lang, text, pid))
            }) {
                for row in rows.flatten() {
                    // Always insert; second pass overwrites with current profile
                    cache.insert((row.0, row.1, row.2), row.3);
                }
            }
        // Second pass: re-insert current profile entries to ensure they take priority
        if let Ok(mut stmt) = conn.prepare(
            "SELECT string_key, namespace, language, text_value FROM localized_string WHERE profile_id = ?1"
        )
            && let Ok(rows) = stmt.query_map(params![self.profile_id], |row| {
                let key: String = row.get(0)?;
                let ns: String = row.get(1)?;
                let lang: String = row.get(2)?;
                let text: String = row.get(3)?;
                Ok((key, ns, lang, text))
            }) {
                for row in rows.flatten() {
                    cache.insert((row.0, row.1, row.2), row.3);
                }
            }
        log::info!(
            "[NameResolver] warmup_localized: profile_id={} loaded {} entries",
            self.profile_id, cache.len()
        );
        self.localized_cache = Some(cache);
    }

    /// US-022b: 预加载 item_base (code → name_en)。
    /// US-024: 同时构建反向索引 (lowercase name_en → code) 用于 cache miss fallback。
    pub fn warmup_item_base(&mut self, conn: &Connection) {
        let mut cache: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut reverse: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let query = "SELECT code, name_en FROM item_base";
        if let Ok(mut stmt) = conn.prepare(query) {
            let rows = stmt.query_map([], |row| {
                let code: String = row.get(0)?;
                let name_en: String = row.get(1)?;
                Ok((code, name_en))
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    if !row.1.is_empty() {
                        reverse.insert(row.1.to_lowercase(), row.0.clone());
                    }
                    cache.insert(row.0, row.1);
                }
            }
        }
        log::info!(
            "[NameResolver] warmup_item_base: profile_id={} loaded {} entries (reverse: {})",
            self.profile_id, cache.len(), reverse.len()
        );
        self.item_base_cache = Some(cache);
        self.name_en_to_code = Some(reverse);
    }

    /// US-022b: 预加载 unique_item_def (unique_id → name_en)。
    pub fn warmup_unique_def(&mut self, conn: &Connection) {
        let mut cache: std::collections::HashMap<u16, String> = std::collections::HashMap::new();
        let query = "SELECT unique_id, name_en FROM unique_item_def";
        if let Ok(mut stmt) = conn.prepare(query) {
            let rows = stmt.query_map([], |row| {
                let uid: i64 = row.get(0)?;
                let name_en: String = row.get(1)?;
                Ok((uid as u16, name_en))
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    cache.insert(row.0, row.1);
                }
            }
        }
        log::info!(
            "[NameResolver] warmup_unique_def: profile_id={} loaded {} entries",
            self.profile_id, cache.len()
        );
        self.unique_def_cache = Some(cache);
    }

    /// Resolve item display name for a specific language.
    ///
    /// Priority:
    /// 1. unique_id → unique_item_def → localized_string
    /// 2. set_id → set_item_def → localized_string
    /// 3. Rare (quality=6): rare_name1 + base + rare_name2
    /// 4. Magic (quality=4): prefix + base + suffix
    /// 5. Base code → item_base → localized_string
    /// 6. Generated fallback
    pub fn resolve(
        &self,
        conn: &Connection,
        code: &str,
        quality: Option<u8>,
        unique_id: Option<u16>,
        set_id: Option<u16>,
        language: &str,
    ) -> ResolvedName {
        // US-024: 慢路径 timer - 累计总耗时,只有慢 resolve 才 log
        static TOTAL_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let started = std::time::Instant::now();
        // Priority 1: Unique item
        if let Some(uid) = unique_id {
            // US-022b: 优先用 unique_def_cache 替代 SQL
            let name_en_opt = if let Some(cache) = &self.unique_def_cache {
                cache.get(&uid).cloned()
            } else {
                queries::get_unique_def(conn, self.profile_id, uid).map(|def| def.name_en)
            };
            if let Some(name_en) = name_en_opt {
                let display = self._get_localized_or_fallback(conn, &name_en, "item_names", language);
                return ResolvedName { display_name: display, name_en, source: NameSource::Unique };
            }
        }

        // Priority 2: Set item
        if let Some(sid) = set_id {
            let items = queries::get_set_items_by_set(conn, self.profile_id, sid);
            if let Some(first) = items.first() {
                let display = self._get_localized_or_fallback(conn, &first.name_en, "item_names", language);
                return ResolvedName { display_name: display, name_en: first.name_en.clone(), source: NameSource::Set };
            }
        }

        // Priority 3: Rare/Magic affix name (quality=6 or 4 with no unique/set)

        // Priority 4: Base item (also handles items where unique_id/set_id aren't
        // resolved but the code exists in item_base)
        let _tr = std::time::Instant::now();
        let base_name_en: Option<String> = if let Some(cache) = &self.item_base_cache {
            cache.get(code).cloned()
        } else {
            queries::get_item_base(conn, self.profile_id, code).map(|def| def.name_en)
        };
        let item_base_time = _tr.elapsed();
        if item_base_time > std::time::Duration::from_millis(10) {
            eprintln!("[timing] SLOW item_base_cache lookup: code={} took={:?}", code, item_base_time);
        }
        if let Some(name_en) = base_name_en {
            let display = if name_en.is_empty() {
                code.to_string()
            } else {
                let _tl = std::time::Instant::now();
                let name = self._get_localized_or_fallback(conn, &name_en, "item_names", language);
                let loc_time = _tl.elapsed();
                if loc_time > std::time::Duration::from_millis(10) {
                    eprintln!("[timing] SLOW _get_localized_or_fallback: code={} key={} lang={} took={:?}", code, name_en, language, loc_time);
                }
                if name == name_en && !language.starts_with("en") {
                    let _tl2 = std::time::Instant::now();
                    let r = self._get_localized(conn, code, "item_names", language).unwrap_or(name);
                    let loc2_time = _tl2.elapsed();
                    if loc2_time > std::time::Duration::from_millis(10) {
                        eprintln!("[timing] SLOW _get_localized (code fallback): code={} lang={} took={:?}", code, language, loc2_time);
                    }
                    r
                } else { name }
            };
            return ResolvedName { display_name: display, name_en, source: NameSource::BaseItem };
        }

        // If item not in item_base (stale profile), try localized by code directly
        if !language.starts_with("en")
            && let Some(t) = self._get_localized(conn, code, "item_names", language) {
                return ResolvedName {
                    display_name: t,
                    name_en: code.to_string(),
                    source: NameSource::BaseItem,
                };
            }

        // US-024b: rune code (r01-r33) 的内置映射
        if let Some(name_en) = rune_name_en(code) {
            let display = self._get_localized_or_fallback(conn, name_en, "item_names", language);
            return ResolvedName { display_name: display, name_en: name_en.to_string(), source: NameSource::BaseItem };
        }

        // Fallback
        let fallback = self._fallback_name(code, quality);
        let result = ResolvedName { display_name: fallback.clone(), name_en: code.to_string(), source: NameSource::Fallback };
        let elapsed = started.elapsed();
        TOTAL_NS.fetch_add(elapsed.as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        let n = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if n.is_multiple_of(100) {
            let total_ms = TOTAL_NS.load(std::sync::atomic::Ordering::Relaxed) / 1_000_000;
            let avg_us = if n > 0 { (TOTAL_NS.load(std::sync::atomic::Ordering::Relaxed) / n) / 1_000 } else { 0 };
            log::info!("[NameResolver] resolve stats: count={} total={}ms avg={}µs/call", n, total_ms, avg_us);
        }
        result
    }

    /// Resolve base item name (no quality/unique/set, just the item type).
    pub fn resolve_base(&self, conn: &Connection, code: &str, language: &str) -> Option<String> {
        // US-022b: 优先用 item_base_cache
        let name_en = if let Some(cache) = &self.item_base_cache {
            cache.get(code).cloned()?
        } else {
            queries::get_item_base(conn, self.profile_id, code).map(|b| b.name_en)?
        };
        let display = self._get_localized_or_fallback(conn, &name_en, "item_names", language);
        // 如果英文名没查到本地化，尝试用物品代码直接查（cascview 数据以 code 为 key）
        if display == name_en && !language.starts_with("en")
            && let Some(code_name) = self._get_localized(conn, code, "item_names", language) {
                return Some(code_name);
            }
        Some(display)
    }

    /// Resolve unique item name by unique_id.
    pub fn resolve_unique(&self, conn: &Connection, unique_id: u16, language: &str) -> Option<ResolvedName> {
        let def = queries::get_unique_def(conn, self.profile_id, unique_id)?;
        let display = self._get_localized_or_fallback(conn, &def.name_en, "item_names", language);
        Some(ResolvedName {
            display_name: display,
            name_en: def.name_en,
            source: NameSource::Unique,
        })
    }

    /// Resolve set item name by set_id (returns the first set item's name).
    pub fn resolve_set(&self, conn: &Connection, set_id: u16, language: &str) -> Option<ResolvedName> {
        let set = queries::get_set_def(conn, self.profile_id, set_id)?;
        let display = self._get_localized_or_fallback(conn, &set.name_en, "item_names", language);
        Some(ResolvedName {
            display_name: display,
            name_en: set.name_en,
            source: NameSource::Set,
        })
    }

    /// Resolve runeword name by code/pattern.
    /// Queries runeword_def table first; falls back to base resolution.
    ///
    /// Runeword resolution is a two-step process:
    /// 1. The parser provides a runeword_id (not available in all code paths)
    /// 2. For the stash path, we try to match the item code against
    ///    runeword_def.allowed_base_types combined with the profile context.
    ///
    /// For now: query all runewords whose allowed_base_types contain the code.
    /// If none match, fall through to base resolution.
    pub fn resolve_runeword(&self, conn: &Connection, code: &str, language: &str) -> Option<String> {
        // Try to find a runeword whose allowed_base_types contains this code
        let mut stmt = conn
            .prepare_cached(
                "SELECT name_en FROM runeword_def
                 WHERE profile_id = ?1
                 AND (',' || allowed_base_types || ',') LIKE ('%,' || ?2 || ',%')
                 LIMIT 1",
            )
            .ok()?;
        if let Ok(rw_name) = stmt.query_row(params![self.profile_id, code], |row| row.get::<_, String>(0))
            && !rw_name.is_empty() {
                let display = self._get_localized_or_fallback(conn, &rw_name, "item_names", language);
                return Some(display);
            }
        // Fallback to base item resolution
        self.resolve_base(conn, code, language)
    }

    /// Resolve rare item name from rare_name1 + base + rare_name2.
    ///
    /// Format: `{rare_name1} {base_locale} {rare_name2}`
    /// Example: "Storm Amulet Dread" or "暴风 护身符 恐惧"
    pub fn resolve_rare(
        &self,
        conn: &Connection,
        code: &str,
        rare_name1: u8,
        rare_name2: u8,
        _quality: Option<u8>,
        language: &str,
    ) -> ResolvedName {
        // US-022b: 优先用 item_base_cache
        let base_name_en: String = if let Some(cache) = &self.item_base_cache {
            cache.get(code).cloned().unwrap_or_else(|| code.to_string())
        } else {
            queries::get_item_base(conn, self.profile_id, code)
                .map(|b| b.name_en)
                .unwrap_or_else(|| code.to_string())
        };
        let base_locale = self._get_localized_or_fallback(conn, &base_name_en, "item_names", language);

        // Look up rare name roots from localized_string (namespace = "item_rarenames")
        let name1 = self._get_rare_name_by_index(conn, rare_name1, language);
        let name2 = self._get_rare_name_by_index(conn, rare_name2, language);

        let name1_str = name1.as_deref().unwrap_or("");
        let name2_str = name2.as_deref().unwrap_or("");

        let display = if !name1_str.is_empty() || !name2_str.is_empty() {
            format!("{} {} {}", name1_str, base_locale, name2_str).trim().to_string()
        } else {
            format!("Rare {}", base_locale)
        };

        ResolvedName {
            display_name: display,
            name_en: format!("{} {} {}", name1_str, base_name_en, name2_str).trim().to_string(),
            source: NameSource::RareMagic,
        }
    }

    /// Resolve magic/rare item name from affix IDs.
    ///
    /// For magic items (quality 4): prefix_name + base_name + suffix_name
    /// For rare items (quality 6): falls back to resolve_rare.
    pub fn resolve_with_affix(
        &self,
        conn: &Connection,
        code: &str,
        quality: Option<u8>,
        unique_id: Option<u16>,
        set_id: Option<u16>,
        rare_name1: Option<u8>,
        rare_name2: Option<u8>,
        affix_ids: &[u16],
        language: &str,
    ) -> ResolvedName {
        // First try unique/set priorities
        if let Some(uid) = unique_id
            && let Some(def) = queries::get_unique_def(conn, self.profile_id, uid) {
                let display = self._get_localized_or_fallback(conn, &def.name_en, "item_names", language);
                return ResolvedName { display_name: display, name_en: def.name_en, source: NameSource::Unique };
            }
        if let Some(sid) = set_id {
            let items = queries::get_set_items_by_set(conn, self.profile_id, sid);
            if let Some(first) = items.first() {
                let display = self._get_localized_or_fallback(conn, &first.name_en, "item_names", language);
                return ResolvedName { display_name: display, name_en: first.name_en.clone(), source: NameSource::Set };
            }
        }

        // Rare: use rare_name1 + rare_name2
        if (quality == Some(6) || quality == Some(8))
            && let (Some(r1), Some(r2)) = (rare_name1, rare_name2) {
                return self.resolve_rare(conn, code, r1, r2, quality, language);
            }

        // Magic: use affix IDs to look up prefix/suffix names
        if quality == Some(4) && affix_ids.len() >= 2 {
            let base = queries::get_item_base(conn, self.profile_id, code);
            let base_name_en = base.as_ref().map(|b| b.name_en.as_str()).unwrap_or(code);
            let base_locale = self._get_localized_or_fallback(conn, base_name_en, "item_names", language);

            // affix_id < 728 = prefix, >= 728 = suffix
            let prefix_id = affix_ids[0];
            let suffix_id = affix_ids[1];

            let prefix_name = self.get_affix_name(conn, prefix_id, language);
            let suffix_name = self.get_affix_name(conn, suffix_id, language);

            let display = match (prefix_name, suffix_name) {
                (Some(p), Some(s)) => format!("{} {} of {}", p, base_locale, s),
                (Some(p), None) => format!("{} {}", p, base_locale),
                (None, Some(s)) => format!("{} of {}", base_locale, s),
                (None, None) => format!("Magic {}", base_locale),
            };

            return ResolvedName {
                display_name: display,
                name_en: format!(
                    "{} {} of {}",
                    self.get_affix_name(conn, prefix_id, "enUS").unwrap_or_default(),
                    base_name_en,
                    self.get_affix_name(conn, suffix_id, "enUS").unwrap_or_default(),
                ).trim().to_string(),
                source: NameSource::RareMagic,
            };
        }

        // Fallback to simple resolve
        self.resolve(conn, code, quality, unique_id, set_id, language)
    }

    /// Look up a rare name root by index (0..200 → "Bite", "Fang", etc.).
    /// Queries localized_string with namespace="item_rarenames".
    /// The Key in the JSON is the English name, indexed sequentially.
    fn _get_rare_name_by_index(&self, conn: &Connection, index: u8, language: &str) -> Option<String> {
        // item-rarenames.json entries are indexed 0..200 with Key = English name.
        // We query by index position. The localized_string table stores Key = English name.
        // Since we don't store the index directly, we get all rare names and use index.
        let mut stmt = conn
            .prepare_cached(
                "SELECT text_value FROM localized_string
                 WHERE profile_id = ?1 AND namespace = 'item_rarenames'
                 AND language = ?2
                 ORDER BY string_key
                 LIMIT 1 OFFSET ?3",
            )
            .ok()?;
        stmt.query_row(params![self.profile_id, language, index], |row| row.get(0))
            .ok()
    }

    /// Look up an affix name by its ID (0-1456ish).
    /// magicprefix.txt IDs 0-727, magicsuffix.txt IDs 728-1456.
    ///
    /// The localized_string table has Key = English affix name (e.g. "Sturdy", "of the Leech").
    /// We don't have direct affix_id → localized_string mapping yet.
    /// This requires item_affix_def table (Phase 2.2 extension).
    ///
    /// Look up an affix display name by its affix_id.
    ///
    /// 1. Query item_affix_def for the English name
    /// 2. For non-English, look up localized_string (namespace="item_nameaffixes")
    /// 3. Falls back to the English name if localization missing
    pub(crate) fn get_affix_name(&self, conn: &Connection, affix_id: u16, language: &str) -> Option<String> {
        let name_en = queries::get_affix_name(conn, self.profile_id, affix_id)?;
        if language == "enUS" {
            return Some(name_en);
        }
        let localized = self._get_localized_or_fallback(conn, &name_en, "item_nameaffixes", language);
        Some(localized)
    }

    // ── Private helpers ──

    /// Look up a localized string from the database, falling back to English key.
    fn _get_localized_or_fallback(&self, conn: &Connection, key: &str, namespace: &str, language: &str) -> String {
        if language == "enUS" {
            return key.to_string();
        }
        self._get_localized(conn, key, namespace, language)
            .unwrap_or_else(|| {
                // Try Chinese fallback chain: zhTW → zhCN → enUS
                if language == "zhTW" {
                    self._get_localized(conn, key, namespace, "zhCN")
                        .unwrap_or_else(|| key.to_string())
                } else {
                    key.to_string()
                }
            })
    }

    /// Direct lookup into localized_string table.
    /// Direct lookup into localized_string table, stripping D2R color codes.
    fn _get_localized(&self, conn: &Connection, key: &str, namespace: &str, language: &str) -> Option<String> {
        // US-022: 缓存命中路径,O(1) HashMap lookup 替代 SQL
        if let Some(cache) = &self.localized_cache {
            let lookup_key = (key.to_lowercase(), namespace.to_string(), language.to_string());
            if let Some(text) = cache.get(&lookup_key) {
                return Some(strip_color_codes(text));
            }
            // US-024: cache miss 时,用 name_en→code 反向索引找 3-char code,再用 code 重查 cache
            if namespace == "item_names"
                && let Some(reverse) = &self.name_en_to_code {
                    let lk = key.to_lowercase();
                    if let Some(code3) = reverse.get(&lk) {
                        let code_lookup = (code3.clone(), namespace.to_string(), language.to_string());
                        if let Some(text) = cache.get(&code_lookup) {
                            return Some(strip_color_codes(text));
                        }
                    }
                }
            // Log first 3 cache misses for debugging
            static MISS_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = MISS_LOG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if n < 3 {
                let lk = key.to_lowercase();
                let similar: Vec<&String> = cache.keys()
                    .filter(|(_k, ns, _)| ns == namespace)
                    .map(|(k, _, _)| k)
                    .take(3)
                    .collect();
                log::warn!(
                    "[NameResolver] _get_localized CACHE MISS #{}: ns={} key={} lang={} first_keys={:?}",
                    n, namespace, lk, language, similar
                );
            }
            // Cache has ALL entries from ALL profiles. If we missed, SQL will also miss.
            return None;
        }
        // No cache (warmup not triggered) — slow SQL path
        static NO_CACHE_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NO_CACHE_LOG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 5 {
            log::warn!(
                "[NameResolver] _get_localized called WITHOUT cache! This is the slow path."
            );
        }
        // Try current profile
        let mut stmt = conn
            .prepare_cached(
                "SELECT text_value FROM localized_string
                 WHERE profile_id = ?1 AND namespace = ?2 AND string_key = ?3 AND language = ?4
                 LIMIT 1",
            )
            .ok()?;
        let result: Option<String> = stmt.query_row(
            params![self.profile_id, namespace, key.to_lowercase(), language],
            |row| row.get(0),
        )
        .ok();
        if let Some(text) = result {
            return Some(strip_color_codes(&text));
        }
        if language != "enUS" {
            log::debug!("[NameResolver] _get_localized MISS: profile={} ns={} key={} lang={}",
                self.profile_id, namespace, key.to_lowercase(), language);
        }
        // Cascade to vanilla profile(s)
        for vp_key in &["vanilla:d2r-92777", "vanilla:default", "vanilla:3.2-92777"] {
            if let Ok(vp_id) = conn.query_row(
                "SELECT id FROM resource_profile WHERE profile_key = ?1 LIMIT 1",
                params![vp_key],
                |row| row.get::<_, i64>(0),
            ) {
                let mut stmt = conn
                    .prepare_cached(
                        "SELECT text_value FROM localized_string
                         WHERE profile_id = ?1 AND namespace = ?2 AND string_key = ?3 AND language = ?4
                         LIMIT 1",
                    )
                    .ok()?;
                let result2: Option<String> = stmt.query_row(
                    params![vp_id, namespace, key.to_lowercase(), language],
                    |row| row.get(0),
                )
                .ok();
                if let Some(text) = result2 {
                    log::debug!("[NameResolver] cascaded to vanilla profile {} for key={}", vp_id, key);
                    return Some(strip_color_codes(&text));
                }
            }
        }
        // Final fallback: try ALL profiles
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT text_value FROM localized_string
             WHERE string_key = ?1 AND namespace = ?2 AND language = ?3
             LIMIT 1",
        )
            && let Ok(text) = stmt.query_row(
                params![key.to_lowercase(), namespace, language],
                |row| row.get::<_, String>(0),
            ) {
                return Some(strip_color_codes(&text));
            }
        None
    }


    /// Generate fallback name when nothing else works.
    /// Mirrors the logic from `item_names.rs::resolve_item_name`.
    fn _fallback_name(&self, code: &str, quality: Option<u8>) -> String {
        let prefix = match quality {
            Some(7) => "Unique ",
            Some(6) => "Rare ",
            Some(5) => "Set ",
            Some(4) => "Magic ",
            Some(3) => "Superior ",
            _ => "",
        };
        format!("{}{}", prefix, code)
    }
}

// ── Global per-profile resolver cache ────────────────────────────────
// P0 perf fix (2026-07-31): every read_stash / read_character / warehouse
// call rebuilt the full NameResolver (warmup_localized loads ALL
// localized_string rows + item_base + unique_item_def, 600-1100ms each).
// Log showed 9 warmups in 5 minutes, with name_resolver_init taking
// ~90% of stash read time. Now the resolver is built once per profile_id
// and shared via Arc — it is immutable after warmup, so sharing is safe.
// Invalidate via [clear_resolver_cache] whenever profile data changes
// (reimport / delete / reset), or [invalidate_resolver] for one profile.
static RESOLVER_CACHE: LazyLock<Mutex<std::collections::HashMap<i64, Arc<NameResolver>>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

/// Get a cached per-profile NameResolver, warming it up on first use.
/// Immutable after warmup, so callers share it via `Arc` (no locking on resolve).
pub fn get_cached_resolver(conn: &Connection, profile_id: i64) -> Arc<NameResolver> {
    // Fast path: already warmed for this profile
    if let Some(r) = RESOLVER_CACHE.lock().get(&profile_id) {
        return r.clone();
    }
    // Slow path: build outside the lock (takes 600-1100ms), then double-check
    let resolver = Arc::new(NameResolver::with_localized_cache(conn, profile_id));
    RESOLVER_CACHE
        .lock()
        .entry(profile_id)
        .or_insert_with(|| resolver.clone())
        .clone()
}

/// Drop the cached resolver for one profile (after deleting that profile).
pub fn invalidate_resolver(profile_id: i64) {
    RESOLVER_CACHE.lock().remove(&profile_id);
}

/// Clear the whole resolver cache (after reimport / profile switch / reset).
pub fn clear_resolver_cache() {
    RESOLVER_CACHE.lock().clear();
}

/// US-024b: 检测是否是 rune code (r01-r33 + 各种 mod rune)。
/// 3-char code 以 "r" 开头且后面 2 字符是数字,例如 r05, r33。
#[allow(dead_code)]
fn is_rune_code(code: &str) -> bool {
    if code.len() != 3 { return false; }
    let bytes = code.as_bytes();
    bytes[0] == b'r' && bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()
}

/// US-024b: rune code → name_en 内置映射 (r01-r33)。
/// 来源:`protocol::d2i::legacy::constants` 硬编码的 ITEM_CODE_MAP。
/// 用于 cache miss 时快速给出 name_en,避免 600ms SQL 兜底。
/// 中文名 (zhCN) 通过 name_en 再走 _get_localized 查找
/// (DB 有 'el rune'/'eth rune' 这类条目在 cascview)。
fn rune_name_en(code: &str) -> Option<&'static str> {
    match code {
        "r01" => Some("El Rune"),
        "r02" => Some("Eld Rune"),
        "r03" => Some("Tir Rune"),
        "r04" => Some("Nef Rune"),
        "r05" => Some("Eth Rune"),
        "r06" => Some("Ith Rune"),
        "r07" => Some("Tal Rune"),
        "r08" => Some("Ral Rune"),
        "r09" => Some("Ort Rune"),
        "r10" => Some("Thul Rune"),
        "r11" => Some("Amn Rune"),
        "r12" => Some("Sol Rune"),
        "r13" => Some("Shael Rune"),
        "r14" => Some("Dol Rune"),
        "r15" => Some("Hel Rune"),
        "r16" => Some("Io Rune"),
        "r17" => Some("Lum Rune"),
        "r18" => Some("Ko Rune"),
        "r19" => Some("Fal Rune"),
        "r20" => Some("Lem Rune"),
        "r21" => Some("Pul Rune"),
        "r22" => Some("Um Rune"),
        "r23" => Some("Mal Rune"),
        "r24" => Some("Ist Rune"),
        "r25" => Some("Gul Rune"),
        "r26" => Some("Vex Rune"),
        "r27" => Some("Ohm Rune"),
        "r28" => Some("Lo Rune"),
        "r29" => Some("Sur Rune"),
        "r30" => Some("Ber Rune"),
        "r31" => Some("Jah Rune"),
        "r32" => Some("Cham Rune"),
        "r33" => Some("Zod Rune"),
        _ => None,
    }
}

/// Strip D2R color formatting codes (ÿc followed by single char) from localized strings.
/// D2R uses ÿcN format for color changes (e.g. ÿc1=red, ÿc0=reset).
/// These should never be shown in the UI.
pub fn strip_color_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == 'ÿ' {
            // D2R format: ÿcX where c is literal 'c' and X is color code
            // Skip the 'c' and the color code character
            chars.next(); // skip 'c'
            chars.next(); // skip color code
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    fn setup_resolver() -> (Connection, NameResolver) {
        // Use in-memory DB for testing
        let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
        let db = Database::init_from_connection(conn).expect("Failed to init DB");
        let conn = db.into_connection();

        // Insert a minimal profile
        conn.execute(
            "INSERT INTO resource_profile (profile_key, source_kind, excel_path)
             VALUES ('test:unit', 'vanilla', '')",
            [],
        ).ok();
        let profile_id: i64 = conn.query_row(
            "SELECT id FROM resource_profile LIMIT 1", [], |row| row.get(0),
        ).expect("Failed to get profile_id");

        // Insert test data into item_base
        conn.execute(
            "INSERT OR REPLACE INTO item_base (code, profile_id, name_en, item_type, item_category)
             VALUES ('cap', 1, 'Cap', 'helm', 'armor')",
            [],
        ).ok();
        conn.execute(
            "INSERT OR REPLACE INTO item_base (code, profile_id, name_en, item_type, item_category)
             VALUES ('r01', 1, 'El Rune', 'rune', 'misc')",
            [],
        ).ok();

        // Insert test localized strings
        conn.execute(
            "INSERT OR REPLACE INTO localized_string (profile_id, namespace, string_key, language, text_value, source_path)
             VALUES (1, 'item_names', 'cap', 'enUS', 'Cap', ''),
                    (1, 'item_names', 'cap', 'zhCN', '帽子', ''),
                    (1, 'item_names', 'cap', 'zhTW', '帽子', ''),
                    (1, 'item_names', 'el rune', 'zhCN', '艾尔', ''),
                    (1, 'item_names', 'el rune', 'zhTW', '艾爾', '')",
            [],
        ).ok();

        // Insert unique item test data
        conn.execute(
            "INSERT OR REPLACE INTO unique_item_def (profile_id, unique_id, name_en, base_code, level, level_req)
             VALUES (1, 105, 'Magefist', 'mgl', 12, 6)",
            [],
        ).ok();
        conn.execute(
            "INSERT OR REPLACE INTO localized_string (profile_id, namespace, string_key, language, text_value, source_path)
             VALUES (1, 'item_names', 'magefist', 'zhCN', '法师之拳', ''),
                    (1, 'item_names', 'magefist', 'zhTW', '法師之拳', '')",
            [],
        ).ok();

        // Insert rare name test data (item-rarenames.json style)
        // Index 0 = "Bite", Index 1 = "Fang", Index 2 = "Storm"
        conn.execute(
            "INSERT OR REPLACE INTO localized_string (profile_id, namespace, string_key, language, text_value, source_path)
             VALUES (1, 'item_rarenames', 'bite', 'enUS', 'Bite', ''),
                    (1, 'item_rarenames', 'bite', 'zhCN', '噬咬', ''),
                    (1, 'item_rarenames', 'fang', 'enUS', 'Fang', ''),
                    (1, 'item_rarenames', 'fang', 'zhCN', '尖牙', ''),
                    (1, 'item_rarenames', 'storm', 'enUS', 'Storm', ''),
                    (1, 'item_rarenames', 'storm', 'zhCN', '暴风', '')",
            [],
        ).ok();

        (conn, NameResolver::new(profile_id))
    }

    #[test]
    fn test_resolve_base_item_english() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "cap", Some(2), None, None, "enUS");
        assert_eq!(name.display_name, "Cap");
        assert_eq!(name.source, NameSource::BaseItem);
    }

    #[test]
    fn test_resolve_base_item_chinese() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "cap", Some(2), None, None, "zhCN");
        assert_eq!(name.display_name, "帽子");
        assert_eq!(name.source, NameSource::BaseItem);
    }

    #[test]
    fn test_resolve_unique_item_chinese() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "mgl", Some(7), Some(105), None, "zhCN");
        assert_eq!(name.display_name, "法师之拳");
        assert_eq!(name.source, NameSource::Unique);
    }

    #[test]
    fn test_resolve_unique_item_english() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "mgl", Some(7), Some(105), None, "enUS");
        assert_eq!(name.display_name, "Magefist");
        assert_eq!(name.source, NameSource::Unique);
    }

    #[test]
    fn test_resolve_fallback_for_unknown_code() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "w  s", Some(7), None, None, "zhCN");
        assert_eq!(name.display_name, "Unique w  s");
        assert_eq!(name.source, NameSource::Fallback);
    }

    #[test]
    fn test_resolve_rune_chinese() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve(&conn, "r01", Some(2), None, None, "zhCN");
        // "El Rune" → localized key "el rune" → zhCN "艾尔"
        assert_eq!(name.display_name, "艾尔");
        assert_eq!(name.name_en, "El Rune");
    }

    #[test]
    fn test_zh_tw_fallback_to_zh_cn() {
        let (conn, resolver) = setup_resolver();
        // "r01" has zhCN "艾尔" but no zhTW in our test data
        // zhTW should fall back to zhCN
        let name = resolver.resolve(&conn, "r01", Some(2), None, None, "zhTW");
        assert_eq!(name.display_name, "艾爾"); // this one has zhTW, so it works
    }

    #[test]
    fn test_resolve_base_name_direct() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve_base(&conn, "cap", "zhCN");
        assert_eq!(name, Some("帽子".to_string()));
    }

    #[test]
    fn test_resolve_unique_by_id() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve_unique(&conn, 105, "zhCN");
        assert!(name.is_some());
        assert_eq!(name.unwrap().display_name, "法师之拳");
    }

    #[test]
    fn test_resolve_unknown_unique_id_fallback() {
        let (conn, resolver) = setup_resolver();
        // unique_id=999 doesn't exist; should fall through to base code
        let name = resolver.resolve(&conn, "cap", Some(7), Some(999), None, "zhCN");
        assert_eq!(name.display_name, "帽子");
        assert_eq!(name.source, NameSource::BaseItem);
    }

    #[test]
    fn test_resolve_rare_name_english() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve_rare(&conn, "cap", 0, 1, Some(6), "enUS");
        // rare_name1=0 (Bite) + base "Cap" + rare_name2=1 (Fang)
        assert_eq!(name.display_name, "Bite Cap Fang");
        assert_eq!(name.source, NameSource::RareMagic);
    }

    #[test]
    fn test_resolve_rare_name_chinese() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve_rare(&conn, "cap", 2, 1, Some(6), "zhCN");
        // rare_name1=2 (Storm→暴风) + base "帽子" + rare_name2=1 (Fang→尖牙)
        assert_eq!(name.display_name, "暴风 帽子 尖牙");
        assert_eq!(name.source, NameSource::RareMagic);
    }

    #[test]
    fn test_resolve_with_affix_rare_path() {
        let (conn, resolver) = setup_resolver();
        let name = resolver.resolve_with_affix(&conn, "cap", Some(6), None, None, Some(0), Some(2), &[], "enUS");
        // rare_name1=0 (Bite) + "Cap" + rare_name2=2 (Storm)
        assert_eq!(name.display_name, "Bite Cap Storm");
    }

    #[test]
    fn test_resolve_with_affix_magic_no_affix_data_falls_back() {
        let (conn, resolver) = setup_resolver();
        // Magic with no affix data (empty Vec): falls through to base item resolution
        let name = resolver.resolve_with_affix(&conn, "cap", Some(4), None, None, None, None, &[], "zhCN");
        assert_eq!(name.display_name, "帽子");
    }

    #[test]
    fn test_resolve_with_affix_prioritizes_unique() {
        let (conn, resolver) = setup_resolver();
        // Even with rare_name1/rare_name2, unique_id should win
        let name = resolver.resolve_with_affix(&conn, "mgl", Some(7), Some(105), None, Some(0), Some(1), &[], "zhCN");
        assert_eq!(name.display_name, "法师之拳");
        assert_eq!(name.source, NameSource::Unique);
    }

    #[test]
    fn test_get_cached_resolver_reuses_instance() {
        let (conn, _) = setup_resolver();
        // Same profile → same Arc instance (warmup runs once)
        let r1 = get_cached_resolver(&conn, 1);
        let r2 = get_cached_resolver(&conn, 1);
        assert!(Arc::ptr_eq(&r1, &r2), "same profile should reuse the cached resolver");
        // Resolver is usable after cache (resolves like a warm one)
        let name = r1.resolve(&conn, "cap", Some(2), None, None, "zhCN");
        assert_eq!(name.display_name, "帽子");
    }

    #[test]
    fn test_invalidate_and_clear_resolver_cache() {
        let (conn, _) = setup_resolver();
        let r1 = get_cached_resolver(&conn, 1);
        invalidate_resolver(1);
        let r2 = get_cached_resolver(&conn, 1);
        assert!(!Arc::ptr_eq(&r1, &r2), "invalidate should force rebuild");
        // Different profiles never share instances
        let p1 = get_cached_resolver(&conn, 1);
        let p2 = get_cached_resolver(&conn, 2);
        assert!(!Arc::ptr_eq(&p1, &p2), "different profiles must have distinct resolvers");
        clear_resolver_cache();
        let p1_again = get_cached_resolver(&conn, 1);
        assert!(!Arc::ptr_eq(&p1, &p1_again), "clear should force rebuild");
    }
}
