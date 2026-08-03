/// POC: 从 .d2i 中提取物品清单，使用游戏数据加载简体中文名
/// fail-fast 模式：解析出错直接报错，不产生垃圾数据
///
/// 用法: cargo test test_poc_game_data -- --nocapture
use std::collections::HashMap;

use d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages;
use d2r_marketplace_lib::protocol::d2i::legacy::item::read_stash_items_from_page;
use d2r_marketplace_lib::protocol::d2i::legacy::item_names::load_item_names;
use d2r_marketplace_lib::protocol::d2i::legacy::item_names::resolve_item_name;

// ─── 游戏数据路径 ──────────────────────────────────────
const MOD_PATH: &str      = r"D:\personal\games\Diablo II Resurrected\mods\D2RMM\D2RMM.mpq\data";
const VANILLA_PATH: &str  = r"D:\dev\d2r\cascview_cn\x64\Work\data\data";
const LANGUAGE: &str      = "zhCN";

/// 从游戏数据路径加载简体中文物品名称（直接解析 JSON，可靠）
fn load_chinese_names_direct() -> HashMap<String, String> {
    use std::io::Read;
    let mut map = HashMap::new();

    let json_paths = [
        (VANILLA_PATH, "item-names.json"),
        (VANILLA_PATH, "item-runes.json"),
        (VANILLA_PATH, "item-gems.json"),
        (MOD_PATH, "item-names.json"),
        (MOD_PATH, "item-runes.json"),
        (MOD_PATH, "item-gems.json"),
    ];

    let lng_dir = "local/lng/strings";
    for &(base, fname) in &json_paths {
        let path = std::path::Path::new(base).join(lng_dir).join(fname);
        if !path.exists() { continue; }
        let mut raw = String::new();
        match std::fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut raw)) {
            Ok(_) => {},
            Err(e) => { eprintln!("  读取失败 {}: {}", path.display(), e); continue; }
        }
        if raw.starts_with('\u{FEFF}') { raw = raw[3..].to_string(); }
        let cleaned: Vec<String> = raw.lines()
            .map(|l| { let t = l.trim_start(); if t.starts_with("//") || t.starts_with("[//") { String::new() } else { l.to_string() } })
            .filter(|l| !l.is_empty())
            .collect();
        let raw = if cleaned.is_empty() { raw } else { cleaned.join("\n") };
        let entries: Vec<serde_json::Value> = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => { eprintln!("  JSON解析失败 {}: {}", path.display(), e); continue; }
        };
        let mut count = 0;
        for entry in &entries {
            let key = entry.get("Key").and_then(|v| v.as_str()).map(|s| s.to_lowercase());
            let zh = entry.get("zhCN").or_else(|| entry.get("enUS")).and_then(|v| v.as_str());
            if let (Some(k), Some(n)) = (key, zh) {
                let mut clean = n.to_string();
                for _ in 0..50 {
                    let bytes = clean.as_bytes();
                    let mut found = false;
                    for i in 0..bytes.len().saturating_sub(2) {
                        if bytes[i] == 0xC3 && bytes[i+1] == 0xBF && bytes[i+2] == b'c' {
                            let remove = (i + 4).min(bytes.len());
                            clean = String::from_utf8_lossy(&bytes[..i]).to_string()
                                + &String::from_utf8_lossy(&bytes[remove..]);
                            found = true;
                            break;
                        }
                    }
                    if !found { break; }
                }
                let clean = clean.trim().to_string();
                if !clean.is_empty() {
                    map.insert(k, clean);
                    count += 1;
                }
            }
        }
        println!("  📄 {:<30} {:>5} 条", fname, count);
    }
    map
}

fn load_chinese_names() -> HashMap<String, String> {
    println!("\n  📚 加载简体中文物品名称\n");
    let mut map = load_item_names(None, LANGUAGE);
    println!("  ℹ️  内置名称表: {} 条 (英文+33条中文)", map.len());
    let zh_map = load_chinese_names_direct();
    let zh_count = zh_map.iter().filter(|(_, v)| v.chars().any(|c| c > '\x7F')).count();
    println!("\n  📊 JSON 直读: 共 {} 条, 含中文 {} 条", zh_map.len(), zh_count);
    let prev = map.len();
    for (k, v) in zh_map { map.insert(k, v); }
    println!("  ✅ 合并后总: {} 条 (+{})\n", map.len(), map.len() - prev);
    for c in &["r01", "r08", "gcv", "jew", "rin", "amu", "cap",
               "7gd", "uit", "utp", "pk1", "toa", "lin", "lyd", "rm1"] {
        let v = map.get(*c).map(|s| s.as_str()).unwrap_or("⚠️  MISS");
        println!("     {:<6} → {}", c, v);
    }
    println!();
    map
}

fn read_jm_count(page_data: &[u8]) -> Option<u16> {
    if page_data.len() < 66 { return None; }
    if &page_data[64..66] != b"JM" { return None; }
    let c = &page_data[66..68];
    Some(u16::from_le_bytes([c[0], c[1]]))
}

/// fixture 可能不随仓库分发（用户本地存档）——缺失时 SKIP。
fn fixture_path(name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures").join(name);
    if p.exists() {
        Some(p)
    } else {
        eprintln!("SKIP: fixture {} 未随仓库分发（本地存档）, 跳过测试", name);
        None
    }
}

#[test]
fn test_poc_game_data() {
    // ═══ Step 1: 中文名称 ────────────────────────────────
    let name_map = load_chinese_names();

    // ═══ Step 2: 解析 .d2i ────────────────────────────────
    let fixture = match fixture_path("ModernSharedStashSoftCoreV2.d2i") { Some(p) => p, None => return };
    let data = std::fs::read(&fixture).expect("读取 .d2i 失败");

    println!("═══════════════════════════════════════════════════");
    println!("  📦 D2I 文件: {}", fixture.display());
    println!("  大小: {} 字节", data.len());
    println!("═══════════════════════════════════════════════════\n");

    let pages = split_legacy_d2i_pages(&data).expect("拆分页面失败");
    println!("📑 共 {} 个页面\n", pages.pages.len());

    // ═══ Step 3: 逐页解析 ────────────────────────────────
    println!("═══════════════════════════════════════════════════");
    println!("  📊 各页面解析结果");
    println!("═══════════════════════════════════════════════════\n");
    println!("  {:<6} {:<8} {:<8} {:<6} {:<10}", "页面", "类型", "JM声明", "有效", "备注");
    println!("  {:<6} {:<8} {:<8} {:<6} {:<10}", "────", "────", "─────", "────", "────");

    for page in &pages.pages {
        let jm = read_jm_count(&page.data).unwrap_or(0);
        match read_stash_items_from_page(page) {
            Ok(items) => {
                println!("  {:<6} {:<8} {:<8} {:<6} ✅  items",
                    format!("#{}", page.index),
                    if page.is_stackable { "📦堆叠" } else { "🎒装备" },
                    if jm > 0 { jm.to_string() } else { "-".to_string() },
                    items.len());
            }
            Err(e) => {
                println!("  {:<6} {:<8} {:<8} {:<6} ❌ {}",
                    format!("#{}", page.index),
                    if page.is_stackable { "📦堆叠" } else { "🎒装备" },
                    if jm > 0 { jm.to_string() } else { "-".to_string() },
                    0, e.chars().take(40).collect::<String>());
            }
        }
    }

    // ═══ Step 4: 页面 #0 第1页 详细清单 ──────────────────
    println!("\n═══════════════════════════════════════════════════");
    println!("  📄 页面 #0 第1页（装备页）");
    println!("═══════════════════════════════════════════════════\n");

    let items0 = read_stash_items_from_page(&pages.pages[0]).unwrap_or_default();
    print_item_table(&items0, &name_map);

    // ═══ Step 5: 堆叠页详情 ────────────────────────────
    for page in &pages.pages {
        if page.is_stackable {
            let items = read_stash_items_from_page(page).unwrap_or_default();
            if !items.is_empty() {
                println!("\n═══════════════════════════════════════════════════");
                println!("  📦 堆叠页 #{}（{}物品）", page.index, items.len());
                println!("═══════════════════════════════════════════════════\n");
                println!("  {:<4} {:<20} {:<32} {:<8}", "#", "代码", "简体中文名", "数量");
                println!("  {:<4} {:<20} {:<32} {:<8}", "──", "────", "────────────", "────");
                for (i, item) in items.iter().enumerate() {
                    let cn = resolve_item_name(&item.item_type, item.quality, &name_map)
                        .chars().take(30).collect::<String>();
                    println!("  {:<4} {:<20} {:<32} {:<8}",
                        i, item.item_type.trim(), cn, item.amount);
                }
            }
        }
    }
}

fn print_item_table(items: &[d2r_marketplace_lib::protocol::d2i::legacy::item::StashItem], map: &HashMap<String, String>) {
    if items.is_empty() { println!("  （无有效物品）\n"); return; }
    println!("  {:<4} {:<20} {:<32} {:<8} {:<8}", "#", "代码", "简体中文名", "数量", "品质");
    println!("  {:<4} {:<20} {:<32} {:<8} {:<8}", "──", "────", "────────────", "────", "────");
    for (i, item) in items.iter().enumerate() {
        let cn = resolve_item_name(&item.item_type, item.quality, map)
            .chars().take(30).collect::<String>();
        let q = match item.quality {
            Some(1) => "低质", Some(2) => "普通", Some(3) => "超强",
            Some(4) => "魔法", Some(5) => "套装", Some(6) => "稀有",
            Some(7) => "暗金", Some(8) => "手工", _ => "-",
        };
        println!("  {:<4} {:<20} {:<32} {:<8} {:<8}",
            i, item.item_type.trim(), cn, item.amount, q);
    }
}

#[test]
fn test_poc_page3_only() {
    unsafe { std::env::set_var("RUST_LOG", "trace"); }
    // _ = env_logger::try_init();
    
    let path = r"D:\work_space\personal_workspace\d2r\ModernSharedStashSoftCoreV2.d2i";
    if !std::path::Path::new(path).exists() { eprintln!("SKIP: 本地 stash 缺失"); return; }
    let data = std::fs::read(path).unwrap();
    let pages = split_legacy_d2i_pages(&data).unwrap();
    
    if pages.pages.len() > 3 {
        let page = &pages.pages[3];
        println!("\n=== 页面 #3 ===");
        println!("偏移: {}, 大小: {}, 堆叠: {}", page.offset, page.size, page.is_stackable);
        
        // JM 头
        let jm = &page.data[64..66];
        let count = u16::from_le_bytes([page.data[66], page.data[67]]);
        println!("JM: {} {}, count={}", jm[0], jm[1], count);
        
        // 尝试解析
        let items = d2r_marketplace_lib::protocol::d2i::legacy::item::read_stash_items_from_page(page).unwrap_or_default();
        println!("解析: {} 个物品", items.len());
        for item in &items {
            println!("  code='{}' amount={} quality={:?} simple={}", 
                item.item_type.trim(), item.amount, item.quality, item.simple_item);
        }
    }
}
