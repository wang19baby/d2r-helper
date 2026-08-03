//! 显示/格式化函数

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use d2r_marketplace_lib::data::affix_names::{MAGIC_PREFIX_NAMES, MAGIC_SUFFIX_NAMES};
use d2r_marketplace_lib::data::items_base::{armor_stats, weapon_stats};
use d2r_marketplace_lib::data::quests_zh::QUEST_NAMES_ZH;
use d2r_marketplace_lib::data::skills_zh::{CLASS_SKILLS, CLASS_SKILL_OFFSETS};
use d2r_marketplace_lib::data::waypoints_zh::WP_NAMES_ZH;
use d2r_marketplace_lib::protocol::d2i::parser::ParsedItem;
use d2r_marketplace_lib::protocol::d2s::attributes::AttributeId;
use d2r_marketplace_lib::protocol::d2s::{
    CharacterClass, D2SCharacter, SkillEntry, W4DialogData, WaypointSet, WooQuestData,
};

use crate::display_names::{item_name_en, item_name_zh, skill_name_zh, slot_name_py};
use crate::helpers;

type SkillGrid = BTreeMap<u8, BTreeMap<u8, BTreeMap<u8, (String, u16)>>>;

// ── Constants ──
const DIFFICULTY_NAMES: [&str; 3] = ["Normal", "Nightmare", "Hell"];
const ACT_NAMES: [&str; 5] = ["ActI", "ActII", "ActIII", "ActIV", "ActV"];
/// Quest bits that count as "completed"
const QF_COMPLETED: u16 = 1 | (1 << 13) | (1 << 14) | (1 << 15);
const CLASS_COUNT: usize = 30;

/// Belt labels
const BELT_LABELS: &[(&str, &str)] = &[
    ("hp1", "小红"), ("hp2", "中红"), ("hp3", "大红"), ("hp4", "超红"), ("hp5", "终红"),
    ("mp1", "小蓝"), ("mp2", "中蓝"), ("mp3", "大蓝"), ("mp4", "超蓝"), ("mp5", "终蓝"),
    ("rvl", "大紫"), ("rvs", "小紫"),
    ("vps", "精力"),
    ("tsc", "传送"), ("isc", "辨识"), ("key", "钥匙"),
    ("yps", "解毒"), ("wms", "融冰"),
];

/// Quest display: D2SLib data index → QUEST_NAMES_ZH display order.
/// Data entries not in the quest log (Introduction, Completion, Extra*) are skipped.
const QUEST_DATA_INDICES: [&[usize]; 5] = [
  &[1, 2, 3, 5, 4, 6],  // ActI:   邪恶洞穴(1), 修女埋骨地(2), 黑暗森林(3), 遗忘之塔(5), 救赎(4), 姐妹之情(6)
  &[1, 2, 4, 5, 6, 7],  // ActII:  下水道(1), 死亡神殿(2), 神秘避难所(4), 召唤者(5), 七个古墓(6), 塔拉夏的古墓(7)
  &[4, 3, 1, 2, 5, 6],  // ActIII: 黄金鸟(4), 刃爪魔(3), 蓝·伊森之书(1), 克林姆的意志(2), 意志之力(5), 吞噬者(6)
  &[1, 2, 3],           // ActIV:  堕落的天使(1), 地狱之锻(2), 恐怖终结(3)
  &[3, 4, 5, 6, 8, 7],  // ActV:   哈洛加斯围城战(3), 亚瑞特山救援(4), 冰之囚(5), 堕落者(6), 世界之石要塞(8), 远古之路(7)
];

// ═══════════════════════════════════════════════
// Display: Header
// ═══════════════════════════════════════════════

pub(crate) fn show_header(info: &D2SCharacter, _path: &Path) -> String {
    let h = &info.header;
    let a = &info.attributes;
    let cn = CharacterClass::from(h.class).name_cn();
    let exp_raw = a.get(AttributeId::Experience);
    let ts = h.save_timestamp;
    let created = if ts != 0 && ts != u32::MAX {
        let secs = ts as i64;
        let nanos = 0u32;
        match chrono::DateTime::from_timestamp(secs, nanos) {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => "Unknown".to_string(),
        }
    } else {
        "Unknown".to_string()
    };
    let hc = if h.is_hardcore() { " HC" } else { "" };
    let dead = if h.is_dead() { " [已死亡]" } else { "" };
    let exp = if h.is_expansion() { " 资料片" } else { "" };
    let lvl = a.get(AttributeId::Level);
    let weapon_set = if h.active_weapon == 0 { "主手" } else { "副手" };

    // 当前位置 (from woo.progression)
    let prog = info.woo.progression;
    let cur_diff_idx = (prog / 5).min(2) as usize;
    let cur_act_idx = (prog % 5).min(4) as usize;
    let diff_names = ["Normal", "Nightmare", "Hell"];
    let act_roman = ["I", "II", "III", "IV", "V"];

    let mut lines = Vec::new();
    lines.push(format!("{:<8}: {}", "角色名", h.name));
    lines.push(format!("{:<8}: {} (id={})", "职业", cn, h.class));
    lines.push(format!("{:<8}:{}{}{}", "状态", hc, dead, exp));
    lines.push(format!("{:<8}: {}", "等级", lvl));
    lines.push(format!("{:<8}: {}", "武器", weapon_set));
    lines.push(format!("{:<8}: {}", "经验值", helpers::fmt_num(exp_raw)));
    lines.push(format!("{:<8}: {} Act {}", "当前位置", diff_names[cur_diff_idx], act_roman[cur_act_idx]));
    lines.push(format!("{:<8}: {}", "最后保存", created));
    lines.join("\n")
}

// ═══════════════════════════════════════════════
// Display: Woo! Quests
// ═══════════════════════════════════════════════

pub(crate) fn show_woo(woo: &WooQuestData) -> String {
    if woo.difficulties.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push("── 任务进度 ──".to_string());
    for (di, diff_name) in DIFFICULTY_NAMES.iter().enumerate() {
        if di >= woo.difficulties.len() { break; }
        lines.push(format!("  [{}]", diff_name));
        for (ai, act_name) in ACT_NAMES.iter().enumerate() {
            if ai >= woo.difficulties[di].len() { break; }
            let act = &woo.difficulties[di][ai];
            let indices = QUEST_DATA_INDICES.get(ai).copied().unwrap_or(&[]);
            let qnames = if ai < 5 { QUEST_NAMES_ZH[ai].as_slice() } else { &[""; 6] };
            let completed = indices.iter().filter(|&&di| di < act.len() && (act[di] & QF_COMPLETED) != 0).count();
            let mut quest_strs = Vec::new();
            for (ni, &di) in indices.iter().enumerate() {
                if di >= act.len() { break; }
                let qn = if ni < qnames.len() { qnames[ni] } else { "" };
                let mark = if (act[di] & QF_COMPLETED) != 0 { "✓" } else { "✗" };
                quest_strs.push(format!("{}{}", mark, qn));
            }
            lines.push(format!("    {} {}/{}  {}", act_name, completed, indices.len(), quest_strs.join(" ")));
        }
    }
    lines.join("\n")
}

// ═══════════════════════════════════════════════
// Display: WS Waypoints
// ═══════════════════════════════════════════════

pub(crate) fn show_ws(ws: &WaypointSet) -> String {
    let all_wp = [&ws.normal, &ws.nightmare, &ws.hell];
    let mut all_empty = true;
    for wp in &all_wp {
        if wp.iter().any(|&b| b) { all_empty = false; break; }
    }
    if all_empty { return String::new(); }

    let act_wp_counts: [usize; 5] = [9, 9, 9, 3, 9];

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push("── 小站 ──".to_string());
    for (di, diff_name) in DIFFICULTY_NAMES.iter().enumerate() {
        let wp = all_wp[di];
        lines.push(format!("  [{}]", diff_name));
        let mut offset = 0;
        for (ai, act_name) in ACT_NAMES.iter().enumerate() {
            let count = act_wp_counts[ai];
            if offset + count > wp.len() { break; }
            let mut parts = Vec::new();
            for bi in 0..count {
                let mark = if wp[offset + bi] { "✓" } else { "✗" };
                let name = if ai < 5 && bi < WP_NAMES_ZH[ai].len() { WP_NAMES_ZH[ai][bi] } else { "" };
                parts.push(format!("{}{}", mark, name));
            }
            offset += count;
            lines.push(format!("    {}: {}", act_name, parts.join(" ")));
        }
    }
    lines.join("\n")
}

pub(crate) fn show_w4(_w4: &W4DialogData, woo: &WooQuestData) -> String {
    // NPC rewards determined by quest completion, not W4 extra bits
    // (act, data_index, label_zh)
    let reward_npcs: [(usize, usize, &str); 3] = [
        (0, 1, "Charsi打造"),    // Act I Den of Evil
        (4, 4, "Anya命名"),      // Act V Rescue on Mount Arreat
        (4, 3, "Qualkehk打孔"),   // Act V Siege on Harrogath
    ];

    // Helper: check if a specific quest data entry has completion flags set
    let quest_done = |di: usize, act: usize, data_idx: usize| -> bool {
        woo.difficulties.get(di)
            .and_then(|acts| acts.get(act))
            .map(|q| q.len() > data_idx && (q[data_idx] & QF_COMPLETED) != 0)
            .unwrap_or(false)
    };

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push("── NPC 奖励 ──".to_string());
    for (di, diff_name) in DIFFICULTY_NAMES.iter().enumerate() {
        lines.push(format!("  [{}]", diff_name));
        for &(act, data_idx, label) in &reward_npcs {
            if quest_done(di, act, data_idx) {
                lines.push(format!("    可用 {}", label));
            } else {
                lines.push(format!("    未解锁 {}", label));
            }
        }
        // Respec (Akara): Den of Evil 完成后可重置属性/技能点
        if quest_done(di, 0, 1) {
            lines.push("    可用 重置属性/技能点".to_string());
        } else {
            lines.push("    未解锁 重置属性/技能点（需完成邪恶洞穴）".to_string());
        }
    }
    lines.join("\n")
}

// ═══════════════════════════════════════════════
// Display: Skills
// ═══════════════════════════════════════════════

pub(crate) fn show_skills(skills: &[SkillEntry], class_id: u8) -> String {
    if skills.is_empty() {
        return String::new();
    }
    let mut levels = [0u16; CLASS_COUNT];
    for se in skills {
        if (se.id as usize) < CLASS_COUNT {
            levels[se.id as usize] = se.level;
        }
    }
    if levels.iter().all(|&l| l == 0) {
        return String::new();
    }

    let tree_names_all: [&[&str; 3]; 8] = [
        &["弓术", "被动&魔法", "标枪"],
        &["火焰", "闪电", "冰霜"],
        &["召唤", "毒素&白骨", "诅咒"],
        &["战斗技能", "防御灵气", "攻击灵气"],
        &["战斗技能", "战斗专家", "战吼"],
        &["召唤", "变形", "元素"],
        &["武学", "影子训练", "陷阱"],
        &["召唤/仆从", "咒术/邪能", "符印/毁灭"],
    ];
    let tree_names = if (class_id as usize) < tree_names_all.len() {
        tree_names_all[class_id as usize]
    } else {
        &["树1", "树2", "树3"]
    };
    let mut grid: SkillGrid = BTreeMap::new();
    let base_id = *CLASS_SKILL_OFFSETS.get(class_id as usize).unwrap_or(&0);
    for (idx, &lvl) in levels.iter().enumerate() {
        let global_id = base_id + idx as u16;
        // 找树位置：跳过 pg=0（Python skilldesc.txt 无此职业数据时的 fallback）
        for &(gid, name, pg, rw, cl) in CLASS_SKILLS {
            if gid == global_id {
                if pg > 0 {
                    grid.entry(pg).or_default()
                        .entry(rw).or_default()
                        .insert(cl, (name.to_string(), lvl));
                }
                break;
            }
        }
    }

    if grid.is_empty() {
        // Fallback: list active skills
        let mut lines = Vec::new();
        lines.push(String::new());
        lines.push("── 技能 ──".to_string());
        for (idx, &lvl) in levels.iter().enumerate() {
            if lvl > 0 {
                let bar = "█".repeat(std::cmp::min(lvl as usize, 25));
                lines.push(format!("  {} {} {}", helpers::skill_name(idx, class_id), bar, lvl));
            }
        }
        if lines.len() <= 2 { return String::new(); }
        return lines.join("\n");
    }

    const COL_W: usize = 28;
    const BAR_MAX: usize = 12;
    const SEP: &str = " │ ";

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push("── 技能树 ──".to_string());
    for (pg, rows) in &grid {
        let title = if (*pg as usize - 1) < tree_names.len() { tree_names[*pg as usize - 1] } else { "树" };
        lines.push(format!("  ── {} ──", title));
        let max_row = *rows.keys().last().unwrap_or(&1);
        for rw in 1..=max_row {
            let cols = rows.get(&rw);
            let mut cells = Vec::new();
            for cl in 1..=3 {
                let txt = match cols.and_then(|c| c.get(&cl)) {
                    Some((name, lvl)) => {
                        let bar = "█".repeat(std::cmp::min(*lvl as usize, BAR_MAX));
                        let label = helpers::cjk_ljust(name, 12);
                        if *lvl > 0 {
                            format!("{} {} {}", label, bar, lvl)
                        } else {
                            format!("{} {}", label, lvl)
                        }
                    }
                    None => String::new(),
                };
                cells.push(helpers::cjk_ljust(&txt, COL_W));
            }
            lines.push(format!("  R{} {}", rw, cells.join(SEP)));
        }
    }
    lines.join("\n")
}

// ═══════════════════════════════════════════════
// Display: Belt
// ═══════════════════════════════════════════════

pub(crate) fn render_belt(items: &[ParsedItem]) -> Vec<String> {
    let cols = 4;
    let mut lines = Vec::new();
    lines.push(format!("    ── 腰带 ({}/{}) ──", items.len(), cols * 4));
    let mut grid: HashMap<(u8, u8), &str> = HashMap::new();
    for it in items {
        let rw = it.item.x / cols as u8;
        let cl = it.item.x % cols as u8;
        if rw < 4 {
            grid.insert((rw, cl), &it.item.code);
        }
    }
    for r in (0..4).rev() {
        let mut cells = Vec::new();
        for c in 0..cols as u8 {
            let code = grid.get(&(r, c)).copied().unwrap_or("");
            let label = BELT_LABELS.iter()
                .find(|(k, _)| *k == code)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| if code.is_empty() { "··" } else { &code[..std::cmp::min(2, code.len())] });
            cells.push(label);
        }
        lines.push(format!("  {}  {}", r + 1, cells.join(" ")));
    }
    lines.push("       1    2    3    4".to_string());
    lines
}

// ═══════════════════════════════════════════════
// Display: single item line helpers
// ═══════════════════════════════════════════════

// 4 个 name lookup 函数 (item_name_en, skill_name_zh, slot_name_py, item_name_zh)
// 拆到 display_names.rs (Sprint 2 W8)。
// 本文件保留 render 编排 + quest/waypoint/skill/belt/item 显示函数。

// ═══════════════════════════════════════════════
// Display: single item
// ═══════════════════════════════════════════════

pub(crate) fn emit_item(it: &ParsedItem, _detail: bool) -> Vec<String> {
    let mut lines = Vec::new();

    let qs = helpers::quality_name(it.item.quality);
    let cat = helpers::item_category(&it.item.code);
    let typ = helpers::item_type_str(&it.item);
    let en_name = item_name_en(&it.item.code);
    let zh_name = item_name_zh(&it.item.code).unwrap_or("");
    let np = if !zh_name.is_empty() {
        format!("{}({})[{}]", zh_name, en_name, it.item.code)
    } else {
        format!("{}({})[{}]", it.item.code, en_name, it.item.code)
    };
    let nw = helpers::disp_width(&np);
    let pad = if 46 > nw { 46 - nw } else { 1 };
    lines.push(format!("    {}{} {}·{}·{}", np, " ".repeat(pad), qs, cat, typ));

    // Info line: position, defense, durability, sockets, damage, requirements, runeword
    let mut info = Vec::new();
    if it.item.mode as u8 == 1 {
        info.push(format!("{},({},{})", slot_name_py(it.item.x), it.item.x, it.item.y));
    } else if it.item.x != 0 || it.item.y != 0 {
        info.push(format!("({},{})", it.item.x, it.item.y));
    }
    let eth = it.item.flags.raw & (1 << 22) != 0;
    // Defense (armor)
    if helpers::is_armor(&it.item.code) {
        info.push(format!("防御:{}", helpers::base_defense_val(&it.item.code, eth)));
    }
    if it.item.max_durability > 0 {
        info.push(format!("耐久:{}/{}", it.item.current_durability, it.item.max_durability));
    }
    if it.item.num_sockets > 0 {
        info.push(format!("孔:{}", it.item.num_sockets));
    }
    // Damage
    let dmg = helpers::base_damage_str(&it.item.code, eth);
    for d in &dmg { info.push(d.clone()); }
    if eth { info.push("(无形+50%)".to_string()); }
    // Requirements
    if let Some(ws) = weapon_stats(&it.item.code) {
        let mut reqs = Vec::new();
        if ws.reqstr > 0 { reqs.push(format!("力量{}", ws.reqstr)); }
        if ws.reqdex > 0 { reqs.push(format!("敏捷{}", ws.reqdex)); }
        if ws.levelreq > 0 { reqs.push(format!("等级{}", ws.levelreq)); }
        if !reqs.is_empty() { info.push(format!("需求:{}", reqs.join("+"))); }
    } else if let Some(arm) = armor_stats(&it.item.code) {
        let mut reqs = Vec::new();
        if arm.reqstr > 0 { reqs.push(format!("力量{}", arm.reqstr)); }
        if arm.reqdex > 0 { reqs.push(format!("敏捷{}", arm.reqdex)); }
        if arm.levelreq > 0 { reqs.push(format!("等级{}", arm.levelreq)); }
        if !reqs.is_empty() { info.push(format!("需求:{}", reqs.join("+"))); }
    }
    // Quantity for stackable items
    if it.item.amount > 1 {
        info.push(format!("x{}", it.item.amount));
    }
    // Level requirement for gems, runes, and other non-equipment items
    if it.item.item_level > 0 && !helpers::is_armor(&it.item.code) && !helpers::is_weapon(&it.item.code) {
        info.push(format!("需求等级{}", it.item.item_level));
    }
    // Runeword: lookup name from socketed rune codes
    if it.item.flags.raw & (1 << 26) != 0 {
        let rune_codes: Vec<&str> = it.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
        if !rune_codes.is_empty() {
            if let Some(rw_zh) = d2r_marketplace_lib::data::runewords::match_runeword_zh(&rune_codes) {
                info.push(format!("符文之语:{}", rw_zh));
            } else {
                info.push("符文之语".to_string());
            }
        } else {
            info.push("符文之语".to_string());
        }
    }
    if !info.is_empty() {
        lines.push(format!("   ├{}", info.join("  ")));
    }

    // Collect affix IDs + stat display data from stat_lists
    let mut prefix_ids: Vec<u16> = Vec::new();
    let mut suffix_ids: Vec<u16> = Vec::new();
    let mut stat_lines: Vec<String> = Vec::new();
    for sl in &it.item.stat_lists {
        for s in &sl.stats {
            if s.id == 75 && s.value > 0 { prefix_ids.push(s.value as u16); }
            if s.id == 76 && s.value > 0 { suffix_ids.push(s.value as u16); }
            if s.id == 75 || s.id == 76 { continue; }
            if s.id == 107 && s.param > 0 && s.value > 0 {
                let name = skill_name_zh(s.param as u16).unwrap_or("?");
                stat_lines.push(format!("{}+{}", name, s.value));
                continue;
            }
            if (s.id == 116 || s.id == 127) && s.value > 0 {
                stat_lines.push(format!("所有技能+{}", s.value));
                continue;
            }
            if let Some(lbl) = helpers::stat_label(s.id)
                && s.value != 0 {
                    if s.value == 1 && ["无法冰冻", "无法破坏", "命中致盲"].contains(&lbl) {
                        stat_lines.push(lbl.to_string());
                    } else {
                        stat_lines.push(format!("{}{}", lbl, s.value));
                    }
                }
        }
    }
    for sl in &stat_lines {
        lines.push(format!("   │  {}", sl));
    }

    // Debug line with affix names
    let mut dbg = Vec::new();
    dbg.push(format!("ilvl={}", it.item.item_level));
    if it.item.id != 0 { dbg.push(format!("uid=0x{:08X}", it.item.id)); }
    if !prefix_ids.is_empty() {
        let names: Vec<&str> = prefix_ids.iter().filter_map(|id|
            MAGIC_PREFIX_NAMES.iter().find(|(i, _)| *i == *id).map(|(_, n)| *n)
        ).collect();
        if !names.is_empty() { dbg.push(format!("前缀:{}", names.join("+"))); }
    }
    if !suffix_ids.is_empty() {
        let names: Vec<&str> = suffix_ids.iter().filter_map(|id|
            MAGIC_SUFFIX_NAMES.iter().find(|(i, _)| *i == *id).map(|(_, n)| *n)
        ).collect();
        if !names.is_empty() { dbg.push(format!("后缀:{}", names.join("+"))); }
    }
    dbg.push(format!("raw:{}B@0x{:04x}", it.raw_bit_length.div_ceil(8), it.raw_bit_offset / 8));
    lines.push(format!("   └{}", dbg.join("  ")));

    lines
}

// ═══════════════════════════════════════════════
// Display: All items
// ═══════════════════════════════════════════════

pub(crate) fn show_items(info: &D2SCharacter, detail: bool) -> String {
    let total = info.equipped.len() + info.belt.len() + info.backpack.len()
        + info.cube.len() + info.merc.len();
    if total == 0 {
        return String::new();
    }
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push("── 物品 ──".to_string());
    lines.push(format!("  总数: {}", total));

    let equipped: Vec<&ParsedItem> = info.equipped.iter()
        .filter(|it| {
            if it.is_socketed_subitem { return false; }
            // Skip runes
            if it.item.code.starts_with('r') && it.item.code.len() > 1
                && it.item.code[1..].chars().all(|c| c.is_ascii_digit()) { return false; }
            // Skip gems (codes starting with 'g' that are not armor)
            if it.item.code.starts_with('g') && it.item.code.len() == 3 && !helpers::is_armor(&it.item.code) { return false; }
            true
        })
        .collect();
    if !equipped.is_empty() {
        lines.push(format!("  [身上] {} 件装备:", equipped.len()));
        for it in &equipped { lines.extend(emit_item(it, detail)); }
    }
    if !info.belt.is_empty() {
        lines.extend(render_belt(&info.belt));
    }
    if !info.backpack.is_empty() {
        lines.push(format!("  [背包] {} 件物品:", info.backpack.len()));
        for it in &info.backpack { lines.extend(emit_item(it, detail)); }
    }
    if !info.cube.is_empty() {
        lines.push(format!("  [盒子] {} 件物品:", info.cube.len()));
        for it in &info.cube { lines.extend(emit_item(it, detail)); }
    }
    if !info.merc.is_empty() {
        lines.push(format!("  [佣兵] {} 件装备:", info.merc.len()));
        for it in &info.merc { lines.extend(emit_item(it, detail)); }
    }
    if !info.personal_stash.is_empty() {
        lines.push(format!("  [仓库] {} 件物品:", info.personal_stash.len()));
        for it in &info.personal_stash { lines.extend(emit_item(it, detail)); }
    }
    lines.join("\n")
}

// ═══════════════════════════════════════════════
// Render (choose JSON or text output)
// ═══════════════════════════════════════════════

pub(crate) fn render(info: &D2SCharacter, path: &Path, json_mode: bool, detail: bool) {
    let hsh = helpers::file_hash(path);
    if json_mode {
        let obj = serde_json::json!({
            "name": info.header.name,
            "class": CharacterClass::from(info.header.class).name_cn(),
            "class_id": info.header.class,
            "level": info.attributes.get(AttributeId::Level),
            "attributes": {
                "strength": info.attributes.get(AttributeId::Strength),
                "dexterity": info.attributes.get(AttributeId::Dexterity),
                "vitality": info.attributes.get(AttributeId::Vitality),
                "energy": info.attributes.get(AttributeId::Energy),
                "hitpoints": info.attributes.get(AttributeId::Hitpoints),
                "maxhp": info.attributes.get(AttributeId::MaxHp),
                "stamina": info.attributes.get(AttributeId::Stamina),
                "maxstamina": info.attributes.get(AttributeId::MaxStamina),
                "gold": info.attributes.get(AttributeId::Gold),
                "goldbank": info.attributes.get(AttributeId::GoldBank),
                "statpts": info.attributes.get(AttributeId::StatPoints),
                "newskills": info.attributes.get(AttributeId::NewSkills),
            },
            "file_size": std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
            "checksum": info.header.checksum,
            "num_skills": info.header.num_skills,
            "active_weapon": info.header.active_weapon,
            "menu_layout": format!("0x{:08X}", info.header.menu_layout),
            "hash": hsh,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else {
        println!("{}", show_header(info, path));
        println!("SHA-256  : {}", hsh);
        let woo = show_woo(&info.woo);
        if !woo.is_empty() { println!("{}", woo); }
        let ws = show_ws(&info.waypoints);
        if !ws.is_empty() { println!("{}", ws); }
        let w4 = show_w4(&info.w4, &info.woo);
        if !w4.is_empty() { println!("{}", w4); }
        let sk = show_skills(&info.skills_decoded, info.header.class);
        if !sk.is_empty() { println!("{}", sk); }
        let items = show_items(info, detail);
        if !items.is_empty() { println!("{}", items); }
    }
}
// Sprint 2 W8 boulder-relief: cosmetic doc comment
