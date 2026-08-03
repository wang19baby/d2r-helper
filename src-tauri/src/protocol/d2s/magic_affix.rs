//! MagicPrefix / MagicSuffix 词缀解析。
//!
//! 将 affix_id (0-727=prefix, 728-1456=suffix) 映射为实际 stat 条目，
//! 通过 properties.txt 将 mod1code 转换为 stat_id。
//!
//! 数据来源：
//! - 运行时从 `<excel_dir>/MagicPrefix.txt` 等加载（优先）
//! - 硬编码常用 mod1code→stat 映射（后备）

use crate::protocol::common::ItemStat;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

/// 一条词缀修饰（mod1code + param + min-max 范围）。
#[derive(Debug, Clone)]
struct AffixMod {
    code: String,
    param: i32,
    min: i32,
}

/// properties.txt 一条 Prop 定义。
#[derive(Debug, Clone)]
struct PropDef {
    func1: u8,
    stat1: String,
}

/// 词缀解析器。
struct AffixResolver {
    affixes: HashMap<u16, Vec<AffixMod>>,
    props: HashMap<String, PropDef>,
}

// ── 名称→ID 硬编码表 ──
pub fn stat_name_to_id(name: &str) -> Option<u16> {
    match name {
        "strength" => Some(0),
        "energy" => Some(1),
        "dexterity" => Some(2),
        "vitality" => Some(3),
        "maxhp" => Some(7),
        "maxmana" => Some(9),
        "maxstamina" => Some(11),
        "tohit" => Some(19),
        "toblock" => Some(20),
        "mindamage" => Some(21),
        "maxdamage" => Some(22),
        "damagepercent" => Some(25),
        "manarecoverybonus" => Some(27),
        "armorclass" => Some(31),
        "normal_damage_reduction" => Some(34),
        "magic_damage_reduction" => Some(35),
        "damageresist" => Some(36),
        "magicresist" => Some(37),
        "fireresist" => Some(39),
        "lightresist" => Some(41),
        "coldresist" => Some(43),
        "poisonresist" => Some(45),
        "firemindam" => Some(48),
        "firemaxdam" => Some(49),
        "lightmindam" => Some(50),
        "lightmaxdam" => Some(51),
        "magicmindam" => Some(52),
        "magicmaxdam" => Some(53),
        "coldmindam" => Some(54),
        "coldmaxdam" => Some(55),
        "coldlength" => Some(56),
        "poisonmindam" => Some(57),
        "poisonmaxdam" => Some(58),
        "poisonlength" => Some(59),
        "lifedrainmindam" => Some(60),
        "lifedrainmaxdam" => Some(61),
        "manadrainmindam" => Some(62),
        "manadrainmaxdam" => Some(63),
        "velocitypercent" => Some(67),
        "attackrate" => Some(68),
        "durability" => Some(72),
        "maxdurability" => Some(73),
        "hpregen" => Some(74),
        "item_armor_percent" => Some(16),
        "item_maxhp_percent" => Some(76),
        "item_magicbonus" => Some(80),
        "item_fasterattackrate" => Some(93),
        "item_fastermovevelocity" => Some(96),
        "item_nonclassskill" => Some(97),
        "item_fasterblockrate" => Some(102),
        "item_fastercastrate" => Some(105),
        "item_singleskill" => Some(107),
        "item_normaldamage" => Some(111),
        "item_damagetomana" => Some(114),
        "item_tohit_percent" => Some(119),
        "item_crushingblow" => Some(136),
        "item_openwounds" => Some(135),
        "item_deadlystrike" => Some(141),
        "item_numsockets" => Some(194),
        "item_charged_skill" => Some(204),
        // 套装/词缀扩展 (2026-07-31)
        "item_allskills" => Some(116),
        "item_mana_after_kill" => Some(92),
        "item_fastergethitrate" => Some(99),
        "item_goldbonus" => Some(79),
        "item_lightradius" => Some(89),
        "item_poisonlengthresist" => Some(110),
        "item_thorns_perlevel" => Some(238),
        "item_replenish_quantity" => Some(253),
        "item_maxdamage_perlevel" => Some(17),
        "item_extra_charges" => Some(204),
        _ => None,
    }
}

/// 套装/词缀短 code → stat_id (sets.txt / MagicPrefix 的 prop code)。
/// 2026-07-31: 用于 importer 解析 sets.txt 套装加成。
pub fn prop_code_to_stat_id(code: &str) -> Option<u16> {
    match code {
        "str" => Some(0),
        "enr" | "eng" => Some(1),
        "dex" => Some(2),
        "vit" => Some(3),
        "hp" => Some(7),
        "mana" => Some(9),
        "stam" => Some(11),
        "tohit" => Some(19),
        "bal" => Some(20),
        "mindam" => Some(21),
        "maxdam" => Some(22),
        "dmg%" | "dmg" => Some(25),
        "manarec%" | "manarecovery" => Some(26),
        "manarec" | "manarecoverybonus" => Some(27),
        "ac" => Some(31),
        "ac-hth" => Some(33),
        "red-dmg" | "red-dmg%" => Some(34),
        "red-mag" => Some(35),
        "res-all" | "res-all%" => Some(36),
        "res-mag" => Some(37),
        "res-fire" => Some(39),
        "res-ltng" | "res-light" => Some(41),
        "res-cold" => Some(43),
        "res-pois" => Some(45),
        "res-pois-len" => Some(110),
        "fire-min" => Some(48),
        "fire-max" => Some(49),
        "ltng-min" | "light-min" => Some(50),
        "ltng-max" | "light-max" => Some(51),
        "mag-min" => Some(52),
        "mag-max" => Some(53),
        "cold-min" => Some(54),
        "cold-max" => Some(55),
        "cold-len" | "coldlength" => Some(56),
        "pois-min" | "poison-min" => Some(57),
        "pois-max" | "poison-max" => Some(58),
        "pois-len" | "poisonlength" => Some(59),
        "dmg-lvl" => Some(17),
        "ac%" | "armor%" => Some(16),
        "hp%" => Some(76),
        "mana%" => Some(77),
        "gold%" | "gold" => Some(79),
        "mag%" | "mag" => Some(80),
        "light-rad" | "lightradius" => Some(89),
        "light" => Some(89),
        "att" | "attackrate" => Some(93),
        "swing3" | "swing2" | "swing1" | "swing" => Some(93),
        "balance" => Some(99),
        "block" => Some(102),
        "cast" => Some(105),
        "allskills" => Some(116),
        "skill" => Some(107),
        "charged" => Some(204),
        "mana-kill" => Some(92),
        "heal-kill" => Some(86),
        "dmg-to-mana" => Some(114),
        "thorns" => Some(131),
        "crush" | "crushing" => Some(136),
        "ow" => Some(135),
        "deadly" => Some(141),
        "sock" => Some(194),
        "half-freeze" => Some(144),
        "speed" | "mov" => Some(96),
        "slow" => Some(150),
        "gethit-skill" | "gethit-sklvl" => Some(195),
        "kill-skill" | "kill-sklvl" => Some(198),
        "death-skill" | "death-sklvl" => Some(201),
        "hit-skill" | "hit-sklvl" => Some(195),
        "regen" | "regen-mana" => Some(27),
        "stamregen" | "stamina" => Some(28),
        "dur" => Some(72),
        "maxdur" => Some(73),
        "nofreeze" => Some(167),
        _ => stat_name_to_id(code),
    }
}

/// 硬编码 mod1code → PropDef
fn hardcoded_props() -> HashMap<String, PropDef> {
    let mut m = HashMap::new();
    for (code, stat) in [
        ("str", "strength"), ("dex", "dexterity"), ("vit", "vitality"), ("enr", "energy"),
        ("hp", "maxhp"), ("mana", "maxmana"), ("stam", "maxstamina"),
        ("ac", "armorclass"), ("ac%", "item_armor_percent"),
        ("tohit", "tohit"), ("bal", "toblock"),
        ("dmg%", "damagepercent"), ("mindam", "mindamage"), ("maxdam", "maxdamage"),
        ("red-mag", "magic_damage_reduction"), ("red-dmg", "normal_damage_reduction"),
        ("res-cold", "coldresist"), ("res-fire", "fireresist"),
        ("res-ltng", "lightresist"), ("res-pois", "poisonresist"),
        ("res-mag", "magicresist"), ("res-all", "damageresist"),
        ("swi", "item_fasterblockrate"), ("cast", "item_fastercastrate"),
        ("mov", "item_fastermovevelocity"), ("att", "item_fasterattackrate"),
        ("balance", "item_fastergethitrate"),
        ("dmg-lvl", "item_maxdamage_perlevel"),
        ("hp-lvl", "item_hp_perlevel"), ("mana-lvl", "item_mana_perlevel"),
        ("crush", "item_crushingblow"), ("ow", "item_openwounds"),
        ("deadly", "item_deadlystrike"),
        ("heal-kill", "item_healafterkill"), ("mana-kill", "item_manaafterkill"),
        ("dmg-to-mana", "item_damagetomana"), ("half-freeze", "item_halffreezeduration"),
        ("dem-dmg", "item_demondamage_percent"), ("und-dmg", "item_undeaddamage_percent"),
        ("sock", "item_numsockets"),
    ] {
        m.insert(code.to_string(), PropDef { func1: 1, stat1: stat.to_string() });
    }
    m.insert("skill".to_string(), PropDef { func1: 22, stat1: "item_singleskill".to_string() });
    m.insert("allskills".to_string(), PropDef { func1: 1, stat1: "item_allskills".to_string() });
    m.insert("charged".to_string(), PropDef { func1: 22, stat1: "item_charged_skill".to_string() });
    m
}

// ── 全局解析器实例 ──
static RESOLVER: LazyLock<AffixResolver> = LazyLock::new(build_resolver);

fn build_resolver() -> AffixResolver {
    let (props, affixes) = load_from_disk();
    AffixResolver { affixes, props }
}

/// 从游戏 TXT 文件加载（优先），失败时回退硬编码。
fn load_from_disk() -> (HashMap<String, PropDef>, HashMap<u16, Vec<AffixMod>>) {
    let dir = match resolve_excel_dir() {
        Some(d) => d,
        None => return (hardcoded_props(), HashMap::new()),
    };
    let props = load_properties_txt(&dir).unwrap_or_else(hardcoded_props);
    let affixes = load_affixes(&dir, &props);
    (props, affixes)
}

fn resolve_excel_dir() -> Option<std::path::PathBuf> {
    let candidates = [
        // D2RMM mod game files
        r"D:\personal\games\Diablo II Resurrected\mods\D2RMM\D2RMM.mpq\data\global\excel",
        r"D:\personal\games\Diablo II Resurrected\mods\D2RMM\D2RMM.mpq\data\global\excel\base",
        // Python project's pre-extracted data (used by d2r-zero CLI)
        r"D:\work_space\personal_workspace\d2r-zero\data\test\txt-json",
    ];
    for c in &candidates {
        let p = std::path::PathBuf::from(c);
        if p.join("MagicPrefix.txt").exists() {
            return Some(p);
        }
    }
    None
}

fn load_affixes(dir: &Path, props: &HashMap<String, PropDef>) -> HashMap<u16, Vec<AffixMod>> {
    let mut map = HashMap::new();
    for (fname, base_id) in &[("MagicPrefix.txt", 0u16), ("MagicSuffix.txt", 728u16)] {
        let path = dir.join(fname);
        if !path.exists() { continue; }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            _ => continue,
        };
        let mut entry_idx = 0u16;
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 13 { continue; }
            let name = parts[0].trim();
            if name.is_empty() || name == "0" { continue; }
            let affix_id = base_id + entry_idx;
            entry_idx += 1;
            let mods = parse_mods(&parts, props);
            if !mods.is_empty() {
                map.insert(affix_id, mods);
            }
        }
    }
    map
}


fn parse_mods(fields: &[&str], prop_map: &HashMap<String, PropDef>) -> Vec<AffixMod> {
    let mut mods = Vec::new();
    for i in 1..=3 {
        let ci = match i { 1 => 12, 2 => 16, 3 => 20, _ => break };
        let code = fields.get(ci).unwrap_or(&"").trim();
        if code.is_empty() || !prop_map.contains_key(code) { break; }
        let param = fields.get(ci + 1).unwrap_or(&"").parse::<i32>().unwrap_or(0);
        let min   = fields.get(ci + 2).unwrap_or(&"").parse::<i32>().unwrap_or(0);
        let _max   = fields.get(ci + 3).unwrap_or(&"").parse::<i32>().unwrap_or(0);
        mods.push(AffixMod {
            code: code.to_string(), param, min,
        });
    }
    mods
}

fn load_properties_txt(dir: &Path) -> Option<HashMap<String, PropDef>> {
    for fname in &["properties.txt", "base/properties.txt"] {
        let path = dir.join(fname);
        if !path.exists() { continue; }
        let content = std::fs::read_to_string(path).ok()?;
        let mut props = hardcoded_props();
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.is_empty() || parts[0].is_empty() { continue; }
            let code = parts[0].trim();
            if code.is_empty() { continue; }
            // Python-extracted properties.txt: col[3]=func1 (numeric code), col[4]=stat1 (stat name)
            let func_code = parts.get(3).unwrap_or(&"").trim().parse::<u8>().unwrap_or(1);
            let stat_name = parts.get(4).unwrap_or(&"").trim().to_string();
            if func_code > 0 && !stat_name.is_empty() {
                props.insert(code.to_string(), PropDef { func1: func_code, stat1: stat_name });
            }
        }
        return Some(props);
    }
    None
}

/// 将 affix_id 转换为 ItemStat 列表。
pub fn resolve_affix(affix_id: u16) -> Option<Vec<ItemStat>> {
    let mods = RESOLVER.affixes.get(&affix_id)?;
    let mut stats = Vec::new();
    for affix_mod in mods {
        let prop = RESOLVER.props.get(&affix_mod.code)?;
        match prop.func1 {
            1 => {
                let stat_id = stat_name_to_id(&prop.stat1)?;
                stats.push(ItemStat {
                    id: stat_id, param: 0,
                    value: affix_mod.min as i64,
                    skill_tab: None, skill_level: None,
                    skill_id: None, max_charges: None,
                });
            }
            // func=2: 百分比 stat（同直接应用）
            2 => {
                let stat_id = stat_name_to_id(&prop.stat1)?;
                stats.push(ItemStat {
                    id: stat_id, param: 0,
                    value: affix_mod.min as i64,
                    skill_tab: None, skill_level: None,
                    skill_id: None, max_charges: None,
                });
            }
            22 => {
                let stat_id = stat_name_to_id(&prop.stat1)?;
                stats.push(ItemStat {
                    id: stat_id, param: affix_mod.param as u32,
                    value: affix_mod.min as i64,
                    skill_tab: None, skill_level: None,
                    skill_id: Some(affix_mod.param as u16),
                    max_charges: None,
                });
            }
            // Unknown func codes — treat as direct stat (same as 1/2)
            _ => {
                if let Some(stat_id) = stat_name_to_id(&prop.stat1) {
                    stats.push(ItemStat {
                        id: stat_id, param: 0,
                        value: affix_mod.min as i64,
                        skill_tab: None, skill_level: None,
                        skill_id: None, max_charges: None,
                    });
                }
            }
        }
    }
    Some(stats)
}

/// 检查词缀解析器是否已加载有效数据。
pub fn is_loaded() -> bool {
    !RESOLVER.affixes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix_733_magic_damage_reduction() {
        if !is_loaded() {
            eprintln!("SKIP: 无词缀数据表(需游戏 TXT 或本地文件), 回退硬编码无此 affix");
            return;
        }
        let stats = resolve_affix(733);
        assert!(stats.is_some(), "suffix 733 should resolve");
        if let Some(s) = stats {
            assert!(!s.is_empty());
            assert_eq!(s[0].id, 35, "magic_damage_reduction");
            assert_eq!(s[0].value, 2, "value should match affix data");
        }
    }

    #[test]
    fn test_prefix_1_enhanced_defense() {
        let stats = resolve_affix(1);
        if let Some(s) = stats {
            assert_eq!(s[0].id, 16, "item_armor_percent");
        }
    }
}
