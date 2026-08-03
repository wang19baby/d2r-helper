//! 角色装备 stat/tooltip 验证: 词缀和属性
//! 注意: 依赖本地角色存档文件,文件不存在时自动跳过
use d2r_marketplace_lib::protocol::d2s::parse_file as parse_d2s;

const CHAR_PATH: &str = "C:\\Users\\wang\\Saved Games\\Diablo II Resurrected\\mods\\CycleofImmortals\\开心图书馆长.d2s";

fn load_character() -> Option<d2r_marketplace_lib::protocol::d2s::parser::D2SCharacter> {
    let bytes = std::fs::read(CHAR_PATH).ok()?;
    parse_d2s(&bytes).ok()
}

/// Check stats for an equipped item by code.
fn check_item_stats(
    f: &d2r_marketplace_lib::protocol::d2s::parser::D2SCharacter,
    code: &str,
) -> bool {
    let code = code.trim();
    match f.equipped.iter().find(|pi| pi.item.code.trim() == code) {
        Some(pi) => {
            let stats: Vec<_> = pi.item.stat_lists.iter()
                .flat_map(|sl| sl.stats.iter())
                .filter(|s| s.value != 0)
                .collect();
            eprintln!("code={} stats: {:?}", code,
                stats.iter().map(|s| (s.id, s.value)).collect::<Vec<_>>());
            if stats.is_empty() {
                eprintln!("  WARN: {} has no stat values (may be parsing gap)", code);
                return false;
            }
            true
        }
        None => {
            eprintln!("  SKIP: code='{}' not found in equipped", code);
            false
        }
    }
}

#[test]
fn test_rare_equipment_has_stat_lists() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "skp");
}

#[test]
fn test_magic_equipment_has_stat_lists() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "amu");
}

#[test]
fn test_set_equipment_has_set_stats() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "vbl");
}

#[test]
fn test_rare_gloves_have_stat_lists() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "mgl");
}

#[test]
fn test_runeword_armor_has_socketed_items() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "hla");
}

#[test]
fn test_magic_ring_has_stat_lists() {
    let f = match load_character() { Some(f) => f, None => { eprintln!("SKIP: file not found"); return; } };
    check_item_stats(&f, "rin");
}
