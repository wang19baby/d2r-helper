// Quick debug: dump item_names keys from user's DB to see actual format
use rusqlite::Connection;

fn main() {
    let conn = Connection::open(r"C:\Users\wang\AppData\Local\D2RMarketplace\database\d2r_marketplace.db")
        .expect("open db");
    let mut stmt = conn
        .prepare("SELECT string_key, language, text_value FROM localized_string WHERE namespace='item_names' AND (string_key='r05' OR string_key='el rune' OR string_key='eth rune' OR string_key='eld rune' OR string_key='tir rune' OR string_key LIKE 'rune%') LIMIT 30")
        .expect("prepare");
    let rows = stmt
        .query_map([], |row| {
            let k: String = row.get(0)?;
            let l: String = row.get(1)?;
            let t: String = row.get(2)?;
            Ok((k, l, t))
        })
        .expect("query");
    println!("=== item_names keys (filtered) ===");
    for r in rows.flatten() {
        println!("  key={:?} lang={} text={:?}", r.0, r.1, r.2);
    }
}
