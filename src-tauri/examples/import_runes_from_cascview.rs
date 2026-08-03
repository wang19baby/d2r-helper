//! 从 cascview_cn item-runes.json 导入 rune 中文名到 SQLite database
//!
//! 用法:`cargo run --example import_runes_from_cascview`
//!
//! 来源:D:\dev\d2r\cascview_cn\x64\Work\data\data\local\lng\strings\item-runes.json
//! 目标:`%LOCALAPPDATA%\D2RMarketplace\database\d2r_marketplace.db`
//!
//! 行为:
//! - 解析 33 个 r01-r33 runes 的 enUS/zhCN/zhTW
//! - INSERT INTO item_base (rune code, name_en, item_type='rune')
//! - INSERT INTO localized_string (key='el rune', lang='zhCN', text='艾尔')
//!   (key 是 name_en lowercase,匹配现有 _get_localized 逻辑)

use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;

fn db_path() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join("D2RMarketplace")
        .join("database")
        .join("d2r_marketplace.db")
}

fn cascview_runes_path() -> PathBuf {
    PathBuf::from(r"D:\dev\d2r\cascview_cn\x64\Work\data\data\local\lng\strings\item-runes.json")
}

fn main() {
    let db = db_path();
    println!("[import_runes] DB: {}", db.display());

    let conn = Connection::open(&db).expect("open db");

    // 读 cascview (JSON5-like 格式:含 // 注释,需要 strip)
    let raw = std::fs::read_to_string(cascview_runes_path()).expect("read runes json");
    // 策略:逐字符扫描,跳过 // 开头的整行注释 和 字符串内的 //
    // 简化策略:按行处理,只 skip 行首 // 的行(字符串内的 // 不会在行首)
    let cleaned: String = raw
        .strip_prefix('\u{FEFF}').unwrap_or(&raw)
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n");
    // 文件末尾可能不完整(被 trunc),如果 parse 失败尝试截断到最后一个 `}`
    let cleaned_trimmed = if let Some(last_brace) = cleaned.rfind('}') {
        // 找对应的 [ 开始
        if let Some(first_bracket) = cleaned.find('[') {
            format!("{}{}", &cleaned[first_bracket..=last_brace], "]")
        } else {
            cleaned.clone()
        }
    } else {
        cleaned.clone()
    };
    let arr: Vec<Value> = serde_json::from_str(&cleaned_trimmed)
        .expect("parse json (cascview format may need json5 crate)");
    println!("[import_runes] loaded {} entries from cascview", arr.len());

    // 找到所有 r0X (r01-r33) keys
    let runes: Vec<&Value> = arr.iter()
        .filter(|v| {
            v.get("Key").and_then(|k| k.as_str())
                .map(|s| s.starts_with("r") && s.len() == 3)
                .unwrap_or(false)
        })
        .collect();
    println!("[import_runes] found {} rune entries (r01-r33)", runes.len());

    let mut inserted_item_base = 0;
    let mut inserted_localized = 0;
    let mut skipped = 0;

    // 拿 active profile_id (default = 1)
    let profile_id: i64 = conn
        .query_row(
            "SELECT id FROM resource_profile ORDER BY id LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1);
    println!("[import_runes] using profile_id={}", profile_id);

    for rune in &runes {
        let key = rune["Key"].as_str().unwrap();
        let en = rune["enUS"].as_str().unwrap_or(key);
        let zhcn = rune["zhCN"].as_str().unwrap_or("");
        let zhtw = rune["zhTW"].as_str().unwrap_or("");

        // Skip if no zhCN
        if zhcn.is_empty() || zhcn.starts_with('[') {
            println!("  [skip] {} en={} (no zhCN)", key, en);
            skipped += 1;
            continue;
        }

        // 1. item_base (cache 查找 + reverse index)
        conn.execute(
            "INSERT OR REPLACE INTO item_base (code, profile_id, name_en, item_type, item_category)
             VALUES (?1, ?2, ?3, 'rune', 'misc')",
            rusqlite::params![key, profile_id, en],
        ).expect("insert item_base");
        inserted_item_base += 1;

        // 2. localized_string (zhCN, zhTW) - 用 name_en lowercase 作为 key
        let lk = en.to_lowercase();
        for (lang, text) in [("zhCN", zhcn), ("zhTW", zhtw)] {
            if text.is_empty() { continue; }
            conn.execute(
                "INSERT OR REPLACE INTO localized_string
                 (profile_id, namespace, string_key, language, text_value, source_path)
                 VALUES (?1, 'item_names', ?2, ?3, ?4, 'cascview_cn/item-runes.json')",
                rusqlite::params![profile_id, lk, lang, text],
            ).expect("insert localized");
            inserted_localized += 1;
        }
    }

    println!("[import_runes] DONE:");
    println!("  item_base:    {} inserted", inserted_item_base);
    println!("  localized:    {} inserted", inserted_localized);
    println!("  skipped:      {} (no zhCN)", skipped);
    println!("\n[import_runes] 重启 tauri dev 后,resolve 会用 warmup cache 命中这些条目");
}
