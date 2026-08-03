fn main() {
    let data_path = "D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/global/excel";

    let map_zh = d2r_marketplace_lib::protocol::d2i::legacy::item_names::build_name_map_from_path(data_path, "zhCN");
    let map_en = d2r_marketplace_lib::protocol::d2i::legacy::item_names::build_name_map_from_path(data_path, "enUS");
    let map_tw = d2r_marketplace_lib::protocol::d2i::legacy::item_names::build_name_map_from_path(data_path, "zhTW");
    let map_default = d2r_marketplace_lib::protocol::d2i::legacy::item_names::build_full_name_map();

    println!("═══ 物品名称加载测试 ═══");
    println!("数据路径: {}", data_path);
    println!("zhCN 加载: {} ({} 条)", if map_zh.is_some() { "✅ 成功" } else { "❌ 失败" }, map_zh.as_ref().map_or(0, |m| m.len()));
    println!("enUS 加载: {} ({} 条)", if map_en.is_some() { "✅ 成功" } else { "❌ 失败" }, map_en.as_ref().map_or(0, |m| m.len()));
    println!("zhTW 加载: {} ({} 条)", if map_tw.is_some() { "✅ 成功" } else { "❌ 失败" }, map_tw.as_ref().map_or(0, |m| m.len()));
    println!("内置 加载: {} 条", map_default.len());

    let m = map_zh.unwrap_or_default();
    for (code, en_name) in &[("r01","El Rune"),("r02","Eld Rune"), ("gcv","Chipped Amethyst"), ("rvl","Full Rejuv"), ("pk1","Key of Terror"), ("tes","Essence")] {
        let n = m.get(*code);
        println!("  {} {}: {}", code, en_name, n.unwrap_or(&"(无)".to_string()));
    }

    // Dump all items with Chinese names
    println!("\n所有有中文名的物品 (code != english):");
    let en_map = map_en.unwrap_or_default();
    for (code, zh_name) in &m {
        if let Some(en_name) = en_map.get(code)
            && zh_name != en_name {
                println!("  {}: zhCN={}", code, zh_name);
            }
    }
}
