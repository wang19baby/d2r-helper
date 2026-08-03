//! 辅助函数

use std::path::Path;

use d2r_marketplace_lib::data::items_base::{armor_stats, weapon_stats};
use d2r_marketplace_lib::data::skills_zh::{CLASS_SKILLS, CLASS_SKILL_OFFSETS};
use d2r_marketplace_lib::protocol::common::{Item, ItemQuality};

/// Stat ID to Chinese label
pub(crate) fn stat_label(sid: u16) -> Option<&'static str> {
    Some(match sid {
        0 => "力量+", 1 => "精力+", 2 => "敏捷+", 3 => "体力+",
        6 => "生命+", 7 => "生命上限+", 8 => "法力+", 9 => "法力上限+",
        11 => "耐力+", 16 => "增强防御%", 17|18|25|83 => "增强伤害%",
        19 => "命中+", 21 => "最小伤害", 22 => "最大伤害",
        31 => "防御+", 35 => "魔法伤害减轻", 36 => "物理伤害减轻",
        39 => "火抗%", 41 => "电抗%", 43 => "冰抗%", 45 => "毒抗%",
        76 => "生命上限%", 78 => "攻击反伤",
        79 => "EG%", 80 => "MF%",
        86 => "杀死回血", 87 => "杀死回蓝", 88 => "生命偷取%", 89 => "光照+",
        93 => "攻速%", 94 => "打击恢复%", 95 => "高速跑步%",
        96 => "快速格挡%", 97 => "快速施法%",
        // Stackable item codes
        107 => "技能:", 108 => "怪物安息",
        111 => "物理伤害+", 112 => "命中+", 116 => "所有技能+",
        119 => "生命恢复+", 120 => "法力恢复+",
        127 => "所有技能+", 136 => "压碎打击%",
        137 => "生命偷取%", 138 => "法力偷取%",
        141 => "致命一击%", 142 => "撕开伤口%", 143 => "压碎打击%",
        144 => "冻结目标+", 145 => "命中致盲",
        148 => "毒抗%", 149 => "冰抗%", 150 => "电抗%", 151 => "火抗%",
        99 => "打击恢复%", 105 => "快速施法%",
        156 => "魔法抗性%", 157 => "物抗%",
        161|165 => "MF%", 162|166 => "EG%", 163 => "快速施法%",
        164 => "需求-%", 167 => "无法冰冻", 188 => "单系技能+",
        193 => "凹槽数", 195 => "增强防御%",
        196 => "冰伤最小", 197 => "冰伤最大",
        199 => "火伤最小", 200 => "火伤最大",
        201 => "电伤最小", 202 => "电伤最大", 203 => "毒伤最小",
        214 => "技能:+", 216 => "MF%",
        225 => "每级命中+", 230 => "每级冰抗+",
        259 => "攻击反伤", 261 => "使怪物逃跑%", 286 => "攻速%",
        _ => return None,
    })
}

pub(crate) fn item_category(code: &str) -> &'static str {
    if matches!(code, "hp1" | "hp2" | "hp3" | "hp4" | "hp5" | "mp1" | "mp2" | "mp3" | "mp4" | "mp5" | "rvl" | "rvs" | "vps" | "yps" | "wms") {
        return "药水";
    }
    if code.len() >= 3
        && matches!(&code[..3], "rin" | "amu" | "jew") { return "饰品"; }
    if weapon_stats(code).is_some() { return "武器"; }
    if armor_stats(code).is_some() { return "装备"; }
    if code.starts_with('r') && code[1..].chars().all(|c| c.is_ascii_digit()) { return "符文"; }
    if (code.starts_with('g') || code.starts_with('j')) && code.len() == 3 { return "宝石"; }
    "杂项"
}

pub(crate) fn is_weapon(code: &str) -> bool { weapon_stats(code).is_some() }
pub(crate) fn is_armor(code: &str) -> bool { armor_stats(code).is_some() }

/// CJK-aware display width
pub(crate) fn disp_width(s: &str) -> usize {
    s.chars().map(|c| if c > '\u{2E80}' { 2 } else { 1 }).sum()
}

/// Left-justify with CJK awareness
pub(crate) fn cjk_ljust(s: &str, w: usize) -> String {
    let pad = if disp_width(s) < w { w - disp_width(s) } else { 0 };
    format!("{}{}", s, " ".repeat(pad))
}

pub(crate) fn quality_name(q: ItemQuality) -> &'static str {
    match q {
        ItemQuality::Unique => "暗金",
        ItemQuality::Set => "套装",
        ItemQuality::Rare => "亮金",
        ItemQuality::Magic => "魔法",
        ItemQuality::Superior => "超强",
        ItemQuality::Crafted => "手工",
        ItemQuality::Low => "劣质",
        _ => "普通",
    }
}

pub(crate) fn item_type_str(it: &Item) -> &'static str {
    if it.flags.raw & (1 << 16) != 0 { "E" }
    else if it.flags.raw & (1 << 21) != 0 { "C" }
    else { "F" }
}

pub(crate) fn skill_name(skill_index: usize, class_id: u8) -> String {
    let base_id = *CLASS_SKILL_OFFSETS.get(class_id as usize).unwrap_or(&0);
    let global_id = base_id + skill_index as u16;
    for &(gid, name, _, _, _) in CLASS_SKILLS {
        if gid == global_id {
            return name.to_string();
        }
    }
    format!("skill_{}", global_id)
}

pub(crate) fn file_hash(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    hex::encode(hasher.finalize())
}

/// Format a number with thousands separator
pub(crate) fn fmt_num(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.insert(0, ','); }
        result.insert(0, c);
    }
    result
}

pub(crate) fn base_damage_str(code: &str, ethereal: bool) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(ws) = weapon_stats(code) {
        let mult = if ethereal { 1.5 } else { 1.0 };
        if ws.mindam > 0 {
            let md = (ws.mindam as f64 * mult) as u32;
            let xd = (ws.maxdam as f64 * mult) as u32;
            result.push(format!("攻击:{}-{}", md, xd));
        }
        if ws.twohand_mindam > 0 {
            let md = (ws.twohand_mindam as f64 * mult) as u32;
            let xd = (ws.twohand_maxdam as f64 * mult) as u32;
            result.push(format!("双手:{}-{}", md, xd));
        }
    }
    result
}

pub(crate) fn base_defense_val(code: &str, ethereal: bool) -> u32 {
    armor_stats(code)
        .map(|a| {
            let mult = if ethereal { 1.5 } else { 1.0 };
            (a.minac as f64 * mult) as u32
        })
        .unwrap_or(0)
}
