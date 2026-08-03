//! 角色存档命令（基于 `protocol::d2s`）。
//!
//! 提供最小化角色信息读取（header + attributes + skills + items）。
//!
//! **Layout 兼容策略**:
//! - 标准 D2SLib layout (magic+version+name_length+...+attributes)
//! - 魔改 layout (xieedi.d2s / happy_manman.d2s 观察 — UTF-8 名字 + u16 items count @ 0x103)
//!   此 layout 的 Status/ClassId/Level/Created/LastPlayed 全是 0xff/0x00 噪声,
//!   attributes 段不存在 / 无法解析,equipment 仅按 item code 填 helm/armor/weapon。

use crate::resource::NameResolver;
use crate::protocol::d2s::{
    known_item_bit_layout, parse_file as parse_d2s, D2SCharacter, QuestEntry, SkillEntry, WaypointSet,
    WooQuestData, W4DialogData, ATTRIBUTES_OFFSET,
};
use crate::protocol::common::ItemStat;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use tauri::Emitter;
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use hex;
/// US-018: 物品基础数据查找缓存。
///
/// `build_stored_item_summary` 对每个物品会做:
/// - 1-2 次 `item_size` 线性扫描(~200+ 项)
/// - 2-3 次 `armor_stats` / `weapon_stats` 线性扫描(共 ~500+ 项)
/// - 0-2 次 `match_runeword` 排序+线性扫描
///
/// 在 68 物品场景下,总扫描次数 ~540 次 ×68 ≈ 36000+ 线性扫描。
/// 引入这个 struct 一次性预热,把全部查找降到 O(1)。
pub struct ItemLookupCache {
    sizes: std::collections::HashMap<String, (u8, u8)>,
    armors: std::collections::HashMap<String, Option<crate::data::items_base::ArmorStats>>,
    weapons: std::collections::HashMap<String, Option<crate::data::items_base::WeaponStats>>,
    // US-020: runeword 缓存 — sorted_rune_codes 字符串 → (en_name, zh_name, req_level)
    // 仅对有 socketed 物品的符文之语触发
    runeword_en: std::collections::HashMap<String, Option<String>>,
    runeword_zh: std::collections::HashMap<String, Option<String>>,
    runeword_level: std::collections::HashMap<String, u8>,
}

impl Default for ItemLookupCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemLookupCache {
    pub fn new() -> Self {
        Self {
            sizes: std::collections::HashMap::new(),
            armors: std::collections::HashMap::new(),
            weapons: std::collections::HashMap::new(),
            runeword_en: std::collections::HashMap::new(),
            runeword_zh: std::collections::HashMap::new(),
            runeword_level: std::collections::HashMap::new(),
        }
    }

    /// 预热: 一次性扫描 ITEM_INVENTORY_SIZES / WEAPON_BASE / ARMOR_BASE → HashMap。
    /// 之后所有 lookup 都是 O(1)。
    pub fn warmup(&mut self) {
        for &(k, w, h) in crate::protocol::d2i::legacy::item_sizes::ITEM_INVENTORY_SIZES {
            self.sizes.insert(k.to_string(), (w, h));
        }
        // 注意:armor_stats/weapon_stats 内部会做 `code[1..]`,需要预填两种 key
        for (k, v) in crate::data::items_base::WEAPON_BASE.iter() {
            let (md, xd, h2md, h2xd, rs, rd, lv, lvr, dur, is2h) = *v;
            let ws = crate::data::items_base::WeaponStats {
                mindam: md, maxdam: xd,
                twohand_mindam: h2md, twohand_maxdam: h2xd,
                reqstr: rs, reqdex: rd,
                level: lv, levelreq: lvr,
                durability: dur, is_two_handed: is2h,
            };
            self.weapons.insert((*k).to_string(), Some(ws));
            // 4-char prefix variant
            let k4 = format!("x{}", k);
            self.weapons.insert(k4, Some(ws));
        }
        for (k, v) in crate::data::items_base::ARMOR_BASE.iter() {
            let (mn, mx, rs, rd, lv, lvr, dur) = *v;
            let as_ = crate::data::items_base::ArmorStats {
                minac: mn, maxac: mx,
                reqstr: rs, reqdex: rd,
                level: lv, levelreq: lvr,
                durability: dur,
            };
            self.armors.insert((*k).to_string(), Some(as_));
            let k4 = format!("x{}", k);
            self.armors.insert(k4, Some(as_));
        }
    }

    pub fn item_size(&mut self, code: &str) -> (u8, u8) {
        if let Some(&size) = self.sizes.get(code) {
            return size;
        }
        // 不在 ITEM_INVENTORY_SIZES → 默認 1x1
        (1, 1)
    }

    pub fn armor_stats(&mut self, code: &str) -> Option<crate::data::items_base::ArmorStats> {
        // armor_stats 内部会做 `if code.len() == 4 { &code[1..] } else { code }`
        let key = if code.len() == 4 { &code[1..] } else { code };
        if let Some(cached) = self.armors.get(key) {
            return *cached;
        }
        // 标记为缺失,避免反复 lookup
        self.armors.insert(key.to_string(), None);
        None
    }

    pub fn weapon_stats(&mut self, code: &str) -> Option<crate::data::items_base::WeaponStats> {
        let key = if code.len() == 4 { &code[1..] } else { code };
        if let Some(cached) = self.weapons.get(key) {
            return *cached;
        }
        self.weapons.insert(key.to_string(), None);
        None
    }

    /// US-020: 缓存 runeword lookup。Key = sorted "r01+r02+..."。
    /// 内部用 Vec<String> 作为 key,避免每次都排序+扫描。
    pub fn runeword_en(&mut self, rune_codes: &[&str]) -> Option<String> {
        let key = build_combo_key(rune_codes);
        if let Some(cached) = self.runeword_en.get(&key) {
            return cached.clone();
        }
        let result = crate::data::runewords::match_runeword(rune_codes).map(String::from);
        self.runeword_en.insert(key, result.clone());
        result
    }

    pub fn runeword_zh(&mut self, rune_codes: &[&str]) -> Option<String> {
        let key = build_combo_key(rune_codes);
        if let Some(cached) = self.runeword_zh.get(&key) {
            return cached.clone();
        }
        let result = crate::data::runewords::match_runeword_zh(rune_codes).map(String::from);
        self.runeword_zh.insert(key, result.clone());
        result
    }

    pub fn runeword_req_level(&mut self, rune_codes: &[&str]) -> u8 {
        // 现有 API 接受 &[String],内部也会调用 build_combo
        // 缓存到 rune_codes 字符串
        let key = build_combo_key(rune_codes);
        if let Some(&cached) = self.runeword_level.get(&key) {
            return cached;
        }
        let owned: Vec<String> = rune_codes.iter().map(|s| s.to_string()).collect();
        let result = crate::data::runewords::runeword_req_level(&owned);
        self.runeword_level.insert(key, result);
        result
    }
}

fn build_combo_key(rune_codes: &[&str]) -> String {
    if rune_codes.is_empty() {
        return String::new();
    }
    let mut sorted: Vec<&str> = rune_codes.to_vec();
    sorted.sort();
    sorted.join("+")
}

/// 兼容老调用方: 无缓存版本 (保留供外部 crate 使用)
#[allow(dead_code)]
fn item_size(code: &str) -> (u8, u8) {
    for &(k, w, h) in crate::protocol::d2i::legacy::item_sizes::ITEM_INVENTORY_SIZES {
        if k == code { return (w, h); }
    }
    (1, 1)
}

/// 背包/腰带物品精简摘要（ParsedItem → 前端友好）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredItemSummary {
    pub code: String,
    pub x: u8,
    pub y: u8,
    pub amount: u32,
    pub inv_width: u8,
    pub inv_height: u8,
    pub quality: Option<u8>,
    pub identified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_zh_tw: Option<String>,
    /// D2S container: 1=backpack, 4=cube, 5=stash (from ItemPage)
    pub page: Option<u8>,
    /// 结构化技能/充能词缀（从 stat_lists 中 descfunc=14 / encode=2 / encode=3 提取）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_bonuses: Vec<SkillBonus>,
    /// 物品在 d2s 文件的位偏移（16 进制）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_offset: Option<String>,
    /// 物品位长度
    #[serde(default)]
    pub raw_length: u32,

    // --- Phase 2: Item detail fields ---
    /// 物品等级
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_level: Option<u8>,
    /// 插槽数（孔）
    #[serde(default)]
    pub num_sockets: u8,
    /// 是否无形
    #[serde(default)]
    pub is_ethereal: bool,
    /// 是否符文之语
    #[serde(default)]
    pub is_runeword: bool,
    /// 当前耐久度
    #[serde(default)]
    pub durability_cur: u8,
    /// 最大耐久度
    #[serde(default)]
    pub durability_max: u8,
    /// 基础防御 (盔甲/头盔/盾牌)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_defense: Option<u16>,
    /// 最大防御
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_defense: Option<u16>,
    /// 单手最小伤害
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_damage_1h_min: Option<u16>,
    /// 单手最大伤害
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_damage_1h_max: Option<u16>,
    /// 双手最小伤害
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_damage_2h_min: Option<u16>,
    /// 双手最大伤害
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_damage_2h_max: Option<u16>,
    /// 力量需求
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub req_strength: Option<u8>,
    /// 敏捷需求
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub req_dexterity: Option<u8>,
    /// 等级需求（来自游戏数据，非物品实际 ilvl）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub req_level: Option<u8>,
    /// 镶嵌物品（符文/宝石/珠宝等）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub socketed_items: Vec<StoredItemSummary>,
    /// 原始 stat 分类数据 (base/affix/runeword/set_bonus)
    #[serde(default)]
    pub stats: ItemStats,
    /// 结构化 tooltip 数据
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<TooltipData>,
}

/// 结构化 tooltip 数据（后端分类，前端直接渲染）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_info: Vec<String>,
    /// 扁平词缀行（向后兼容）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stats: Vec<String>,
    /// 基础属性行（白色）：耐久/防御/需求等
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_stats: Vec<String>,
    /// 词缀行（蓝色）：+技能、%MF、抗性等
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affix_stats: Vec<String>,
    /// 符文之语额外行
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runeword_stats: Vec<String>,
    /// 套装加成行
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set_bonus_stats: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_info: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set_info: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sockets: Option<SocketsInfo>,
}

/// 物品的孔位信息(后端 parseItem.num_sockets + socketed_items 推导)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketsInfo {
    /// 总孔数(可能 > socketed_items.len(),未镶嵌的孔位为 None)
    pub count: u8,
    /// 已镶嵌的物品列表(长度 ≤ count)。空位用 length < count 表达,顺序与孔位对齐
    pub items: Vec<SocketedItemInfo>,
}

/// 镶嵌物品的精简展示信息(供 TooltipData.sockets.items 用,
/// 完整数据在 StoredItemSummary.socketed_items 里)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketedItemInfo {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_en: Option<String>,
    pub quality: Option<u8>,
    pub amount: u32,
}

/// 从 flat tooltip_lines 按规则分类到 TooltipData。
pub fn classify_tooltip(lines: &[String]) -> TooltipData {
    let mut base_info = Vec::new();
    let mut stats = Vec::new();
    let mut hidden_info = Vec::new();
    let mut set_info = Vec::new();
    for line in lines {
        if line.starts_with("Code:") || line.starts_with("Type:") || line.starts_with("Quality:") || line.starts_with("Slot:") || line.starts_with("槽位:") || line.starts_with("代码:") || line.starts_with("类型:") || line.starts_with("品质:") || line.starts_with("数量:") || line.starts_with("Quantity:") {
            base_info.push(line.clone());
        } else if line.starts_with("ItemLevel:") || line.starts_with("物品等级:") || line.starts_with("Unique ID:") || line.starts_with("Set ID:") || line.starts_with("Runeword ID:") || line.starts_with("符文之语 ID:") {
            hidden_info.push(line.clone());
        } else if line.contains("Set ID") || line.contains("套装 ID") {
            set_info.push(line.clone());
        } else if line.starts_with("耐久度:") || line.starts_with("Durability:") || line.contains("+") || line.contains("%") || line.chars().any(|c| c.is_ascii_digit()) {
            stats.push(line.clone());
        } else {
            // 名称行等归入 base_info
            base_info.push(line.clone());
        }
    }
    // 把 flat stats 按特征分配到 base_stats / affix_stats（旧路径 fallback）
    let mut base_stats: Vec<String> = Vec::new();
    let mut affix_stats: Vec<String> = Vec::new();
    for l in &stats {
        if l.starts_with("耐久度:") || l.starts_with("防御:") || l.contains("需要") || l.starts_with("攻击力:") || l.starts_with("盾击伤害:") {
            base_stats.push(l.clone());
        } else {
            affix_stats.push(l.clone());
        }
    }
    TooltipData { base_info, stats, base_stats, affix_stats, runeword_stats: Vec::new(), set_bonus_stats: Vec::new(), hidden_info, set_info, sockets: None }
}
/// 前端按 base_stats/affix_stats/runeword_stats/set_bonus_stats 渲染,
/// 无需猜字符串特征。
/// 查询并格式化套装加成行 (绿色 set_bonus_stats)。
/// 存档 set_id = setitems.txt *ID (item_id) → 反查 set 组 id → 查 set_bonus_def。
pub fn append_set_bonuses(
    td: &mut TooltipData,
    conn: &rusqlite::Connection,
    profile_id: i64,
    item_id: u16,
    language: &str,
) {
    let Some(item_def) = crate::resource::queries::get_set_item_by_item_id(conn, profile_id, item_id) else {
        return;
    };
    let bonuses = crate::resource::queries::get_set_bonuses_by_set(conn, profile_id, item_def.set_id);
    if bonuses.is_empty() { return; }
    let lines = crate::resource::TooltipFormatter::format_set_bonuses(&bonuses, language, Some(conn), profile_id);
    for (_, line) in lines {
        if !td.set_bonus_stats.contains(&line) {
            td.set_bonus_stats.push(line);
        }
    }
}

pub fn build_tooltip_from_stats(
    stats: &ItemStats,
    language: &str,
    conn: Option<&rusqlite::Connection>,
    profile_id: i64,
) -> TooltipData {
    // 各分类单独调用 format_stats（避免跨分类合并）
    let base_stats = crate::resource::TooltipFormatter::format_stats(&stats.base, &[], language, conn, profile_id);
    let affix_stats = crate::resource::TooltipFormatter::format_stats(&stats.affix, &[], language, conn, profile_id);
    let rw_stats = crate::resource::TooltipFormatter::format_stats(&stats.runeword, &[], language, conn, profile_id);
    let set_stats = crate::resource::TooltipFormatter::format_stats(&stats.set_bonus, &[], language, conn, profile_id);

    let flat: Vec<_> = base_stats.iter()
        .chain(affix_stats.iter())
        .chain(rw_stats.iter())
        .chain(set_stats.iter())
        .cloned().collect();

    TooltipData {
        base_info: Vec::new(),
        stats: flat,
        base_stats,
        affix_stats,
        runeword_stats: rw_stats,
        set_bonus_stats: set_stats,
        hidden_info: Vec::new(),
        set_info: Vec::new(),
        sockets: None,
    }
}
/// - **skill_charges**: stat 204 (`item_charged_skill`, encode=3)
///   → `+N charges of <Skill>` — `skill_id + skill_level + max_charges + current_charges`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillBonus {
    /// 原始 stat_id (188 / 195-201 / 204)。前端可用作 key 或 debug 显示。
    pub stat_id: u16,
    /// 词缀类型: `"skill_tab"` / `"chance_to_cast"` / `"skill_charges"`
    pub kind: String,
    /// `chance_to_cast` / `skill_charges` 专属: D2R skill id (Blizzard=64, Frozen Orb=..., 等)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<u16>,
    /// `skill_tab` 专属: tab index (0-7, 含义由 class 决定,
    /// 例: 0=Sorceress Cold Spells, 3=Assassin Traps, ...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_tab: Option<u8>,
    /// +N 技能等级（descfunc=14 时为 +N tab levels，encode=2/3 时为 +N skill levels）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_level: Option<u16>,
    /// `chance_to_cast` 专属: 概率百分比（例: 5 → "5% chance to cast"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chance_pct: Option<i64>,
    /// `skill_charges` 专属: 最大充能次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_charges: Option<u8>,
    /// `skill_charges` 专属: 当前充能次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_charges: Option<i64>,
    /// 技能名称（如可用 resolver 则从 DB 解析，否则 null）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
}
/// 按 ItemStat 的拆分字段优先顺序判断 kind:
/// 1. `max_charges.is_some()` → skill_charges (encode=3)
/// 2. `id == 107` → single_skill (encode=1, item_singleskill)
/// 3. `skill_id.is_some()` → chance_to_cast (encode=2)
/// 4. `skill_tab.is_some()` → skill_tab (descfunc=14)
/// 5. 都不是 → 普通 stat, 跳过
pub fn extract_skill_bonuses(
    stats: &[crate::protocol::common::ItemStat],
) -> Vec<SkillBonus> {
    extract_skill_bonuses_inner(stats, None)
}

pub(crate) fn extract_skill_bonuses_with_opt(
    stats: &[crate::protocol::common::ItemStat],
    resolver_opt: Option<(&rusqlite::Connection, &crate::resource::NameResolver)>,
) -> Vec<SkillBonus> {
    extract_skill_bonuses_inner(stats, resolver_opt)
}

fn extract_skill_bonuses_inner(
    stats: &[crate::protocol::common::ItemStat],
    resolver_opt: Option<(&rusqlite::Connection, &crate::resource::NameResolver)>,
) -> Vec<SkillBonus> {
    stats
        .iter()
        .filter_map(|s| {
            if s.max_charges.is_some() {
                Some(SkillBonus {
                    stat_id: s.id,
                    kind: "skill_charges".to_string(),
                    skill_id: s.skill_id,
                    skill_tab: None,
                    skill_level: s.skill_level,
                    chance_pct: None,
                    max_charges: s.max_charges,
                    current_charges: Some(s.value),
                    skill_name: None,
                })
            } else if s.id == 107 {
                let skill_name = resolver_opt.and_then(|(conn, r)| {
                    let sd = crate::resource::queries::get_skill_def(conn, r.profile_id, s.param as u16)?;
                    // Try Chinese localized name
                    let key = format!("{}name", sd.name_en.replace(' ', "").to_lowercase());
                    if let Ok(mut stmt) = conn.prepare_cached(
                        "SELECT text_value FROM localized_string
                         WHERE profile_id = ?1 AND namespace = 'skills' AND string_key = ?2 AND language = 'zhCN'"
                    )
                        && let Ok(row) = stmt.query_row(rusqlite::params![r.profile_id, key], |r| r.get::<_, String>(0)) {
                            return Some(row);
                        }
                    Some(sd.name_en)
                });
                Some(SkillBonus {
                    stat_id: s.id,
                    kind: "single_skill".to_string(),
                    skill_id: s.skill_id,
                    skill_tab: None,
                    skill_level: s.skill_level,
                    chance_pct: None,
                    max_charges: None,
                    current_charges: None,
                    skill_name,
                })
            } else if s.skill_id.is_some() {
                Some(SkillBonus {
                    stat_id: s.id,
                    kind: "chance_to_cast".to_string(),
                    skill_id: s.skill_id,
                    skill_tab: None,
                    skill_level: s.skill_level,
                    chance_pct: Some(s.value),
                    max_charges: None,
                    current_charges: None,
                    skill_name: None,
                })
            } else if s.skill_tab.is_some() {
                Some(SkillBonus {
                    stat_id: s.id,
                    kind: "skill_tab".to_string(),
                    skill_id: None,
                    skill_tab: s.skill_tab,
                    skill_level: s.skill_level,
                    chance_pct: None,
                    max_charges: None,
                    current_charges: None,
                    skill_name: None,
                })
            } else {
                None
            }
        })
        .collect()
}


fn build_stored_item_summary(
    pi: &crate::protocol::d2i::parser::ParsedItem,
    resolver_opt: Option<&(Connection, Arc<NameResolver>)>,
    name_cache: &mut std::collections::HashMap<(String, u8, u16, u16, String), String>,
    cache: &mut ItemLookupCache,
    language: &str,
) -> StoredItemSummary {
    // US-018: 用缓存版本 item_size,O(1) 查找替代线性扫描
    let (w, h) = cache.item_size(&pi.item.code);
    let quality_byte = pi.item.quality.as_u8();
    let quality = Some(quality_byte).filter(|&q| (2..=8).contains(&q));

    // 物品名(三国语言) — 带缓存,避免重复查询同个 code
    let code_map = crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP;
    let code_entry = code_map.iter().find(|(c, _, _, _)| *c == pi.item.code.as_str());
    let (mut name_zh, mut name_en, name_zh_tw) = if let Some((conn, resolver)) = resolver_opt {
        let mut resolve_cached = |code: &str, q: Option<u8>, uid: Option<u16>, sid: Option<u16>, lang: &str| -> String {
            let key = (code.to_string(), q.unwrap_or(0), uid.unwrap_or(0), sid.unwrap_or(0), lang.to_string());
            if let Some(cached) = name_cache.get(&key) {
                return cached.clone();
            }
            let result = resolver.resolve_with_affix(conn, code, q, uid, sid, None, None, &[], lang);
            name_cache.insert(key, result.display_name.clone());
            result.display_name
        };
        let z = resolve_cached(&pi.item.code, quality, pi.item.unique_id, pi.item.set_id, "zhCN");
        let t = resolve_cached(&pi.item.code, quality, pi.item.unique_id, pi.item.set_id, "zhTW");
        let e = resolve_cached(&pi.item.code, quality, pi.item.unique_id, pi.item.set_id, "enUS");
        (Some(z), Some(e), Some(t))
    } else {
        // fallback: ITEM_CODE_MAP 默认是英文
        let en = code_entry.map(|(_, n, _, _)| n.to_string());
        (en.clone(), en.clone(), en)
    };

    // 符文之语名称追加 [隐秘]
    // US-020: 走 cache.runeword_en/zh,首次扫描 + 之后 O(1)
    if pi.item.flags.is_runeword() && !pi.item.socketed_items.is_empty() {
        let rune_codes: Vec<&str> = pi.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
        if let Some(rw_en) = cache.runeword_en(&rune_codes) {
            name_en = Some(format!("{} [{}]", name_en.as_deref().unwrap_or("?"), rw_en));
            if let Some(rw_zh) = cache.runeword_zh(&rune_codes) {
                name_zh = Some(format!("{}[{}]", name_zh.as_deref().unwrap_or("?"), rw_zh));
            }
        }
    }

    // 超强(3)及以上品质才有词缀属性（超强武器有 +准确率/+增强伤害 等）
    // Magic(4+)有完整词缀系统
    let _has_stats = quality_byte >= 3
        && pi.item.stat_lists.iter().any(|sl| !sl.stats.is_empty());
    let _localized_name = name_zh.clone().unwrap_or_default();
    let _en_name = name_en.clone().unwrap_or_default();
    let _kind = code_map.iter()
        .find(|(c, _, _, _)| *c == pi.item.code.as_str())
        .map(|(_, _, k, _)| k.to_string())
        .unwrap_or_else(|| "misc".to_string());
    // 用 build_tooltip_from_stats 替代 stash_tooltip + classify_tooltip
    // US-019: 复用同一个 stats_cat,避免行542 重复 categorize_item_stats 调用
    let stats_cat = categorize_item_stats(pi.item.quality.as_u8(), pi.item.flags.is_runeword(), &pi.item.stat_lists);
    let stats_cat_for_field = stats_cat.clone();
    let (conn_opt, prof_id) = match resolver_opt {
        Some((conn, resolver)) => (Some(conn), resolver.profile_id),
        None => (None, 0),
    };
    let mut classified = build_tooltip_from_stats(&stats_cat, language, conn_opt, prof_id);
    // 底材基础属性（防御、需求）
    // 实际防御优先: item body 解析出的 defense (已含 ED 加成);0 时回退底材区间
    let code = &pi.item.code;
    if let Some(def) = cache.armor_stats(code) {
        if pi.item.defense > 0 {
            classified.base_stats.insert(0, format!("防御: {}", pi.item.defense));
        } else if def.minac > 0 || def.maxac > 0 {
            classified.base_stats.insert(0, format!("防御: {}-{}", def.minac, def.maxac));
        }
        if def.levelreq > 0 { classified.base_stats.push(format!("需要等级: {}", def.levelreq)); }
        if def.reqstr > 0 { classified.base_stats.push(format!("需要力量: {}", def.reqstr)); }
        if def.reqdex > 0 { classified.base_stats.push(format!("需要敏捷: {}", def.reqdex)); }
    }
    // 盾牌 smite 伤害 (含圣骑士盾)
    if let Some((smin, smax)) = crate::data::items_base::shield_smite(code) {
        classified.base_stats.push(if smin == smax {
            format!("盾击伤害: {}", smin)
        } else {
            format!("盾击伤害: {}-{}", smin, smax)
        });
    }
    if let Some(wpn) = cache.weapon_stats(code) {
        let (dmin, dmax) = wpn.display_damage();
        if dmin > 0 || dmax > 0 {
            classified.base_stats.push(if dmin == dmax {
                format!("攻击力: {}", dmin)
            } else {
                format!("攻击力: {}-{}", dmin, dmax)
            });
        }
        if wpn.levelreq > 0 { classified.base_stats.push(format!("需要等级: {}", wpn.levelreq)); }
        if wpn.reqstr > 0 { classified.base_stats.push(format!("需要力量: {}", wpn.reqstr)); }
        if wpn.reqdex > 0 { classified.base_stats.push(format!("需要敏捷: {}", wpn.reqdex)); }
    }
    // 耐久度
    if pi.item.max_durability > 0 {
        classified.base_stats.push(format!("耐久度: {}/{}", pi.item.current_durability, pi.item.max_durability));
    }
    // 物品等级
    if pi.item.item_level > 0 {
        classified.hidden_info.push(format!("物品等级: {}", pi.item.item_level));
    }
    // 符文之语等级需求
    if pi.item.flags.is_runeword() && !pi.item.socketed_items.is_empty() {
        let rune_codes: Vec<&str> = pi.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
        // US-020: 走 cache.runeword_req_level
        let rw_level = cache.runeword_req_level(&rune_codes);
        if rw_level > 0 {
            classified.base_stats.push(format!("需要等级: {}", rw_level));
        }
    }
    // 魔法词缀等级需求
    if pi.item.quality.as_u8() == 4 && (pi.magic_prefix_id.is_some() || pi.magic_suffix_id.is_some()) && let Some(conn) = conn_opt {
            let req = crate::resource::queries::get_magic_item_req_level(
                conn, prof_id, pi.magic_prefix_id, pi.magic_suffix_id);
            if req > 0 {
                classified.base_stats.retain(|l| !l.starts_with("需要等级:"));
                classified.base_stats.push(format!("需要等级: {}", req));
            }
    }
    // 暗金物品等级需求 (unique_item_def.level_req)
    if pi.item.quality.as_u8() == 7
        && let Some(uid) = pi.item.unique_id
        && let Some(conn) = conn_opt
        && let Some(def) = crate::resource::queries::get_unique_def(conn, prof_id, uid)
        && def.level_req > 0 {
            classified.base_stats.retain(|l| !l.starts_with("需要等级:"));
            classified.base_stats.push(format!("需要等级: {}", def.level_req));
    }
    // 套装物品等级需求 (set_item_def.level_req)
    if pi.item.quality.as_u8() == 5
        && let Some(sid) = pi.item.set_id
        && let Some(conn) = conn_opt
        && let Some(def) = crate::resource::queries::get_set_item_by_item_id(conn, prof_id, sid)
        && def.level_req > 0 {
            classified.base_stats.retain(|l| !l.starts_with("需要等级:"));
            classified.base_stats.push(format!("需要等级: {}", def.level_req));
    }
    // 套装加成 (绿色 set_bonus_stats, 来自 sets.txt)
    if pi.item.quality.as_u8() == 5
        && let Some(sid) = pi.item.set_id
        && let Some(conn) = conn_opt {
            append_set_bonuses(&mut classified, conn, prof_id, sid, language);
    }

    // ── Sockets: 孔数 + 镶嵌物品(结构化) ──
    // num_sockets > 0 时填,前端 ItemTooltip 可直接渲染"3 个孔 (r01, r02, r03)"
    if pi.item.num_sockets > 0 {
        let socketed: Vec<SocketedItemInfo> = pi.item.socketed_items.iter().map(|si| {
            let q = si.quality.as_u8();
            let cache_key = (si.code.clone(), q, si.unique_id.unwrap_or(0), si.set_id.unwrap_or(0), language.to_string());
            let cached_name = name_cache.get(&cache_key).cloned();
            let (name_zh, name_en) = match cached_name {
                Some(n) => (Some(n.clone()), Some(n)),
                None => (None, None),
            };
            SocketedItemInfo {
                code: si.code.clone(),
                name_zh,
                name_en,
                quality: Some(q).filter(|&v| (2..=8).contains(&v)),
                amount: si.amount,
            }
        }).collect();
        classified.sockets = Some(SocketsInfo {
            count: pi.item.num_sockets,
            items: socketed,
        });
    }

    // ── Base stats lookup (weapon/armor data) ──
    // US-018: 复用上方已查询的 armor/weapon 结果,避免重复线性扫描
    let code = &pi.item.code;
    let base_def_opt = cache.armor_stats(code)
        .map(|a| (a.minac, a.maxac));
    let dmg_opt = cache.weapon_stats(code)
        .map(|w| (w.mindam, w.maxdam, w.twohand_mindam, w.twohand_maxdam));
    let req_from_wpn = cache.weapon_stats(code)
        .map(|w| (w.reqstr, w.reqdex, w.levelreq));
    let req_from_armor = cache.armor_stats(code)
        .map(|a| (a.reqstr, a.reqdex, a.levelreq));
    let (req_str, req_dex, req_lvl) = req_from_wpn.or(req_from_armor).unwrap_or((0, 0, 0));

    let flags = &pi.item.flags;

    StoredItemSummary {
        code: code.clone(),
        x: pi.item.x,
        y: pi.item.y,
        amount: pi.item.amount,
        inv_width: w,
        inv_height: h,
        quality,
        identified: flags.identified(),
        name_zh,
        name_en,
        name_zh_tw,
        page: pi.item.page.map(|p| p.as_u8()),
        raw_offset: Some(format!("0x{:04X}", pi.raw_bit_offset / 8)),
        raw_length: pi.raw_bit_length as u32,

        item_level: if pi.item.item_level > 0 { Some(pi.item.item_level) } else { None },
        num_sockets: pi.item.num_sockets,
        is_ethereal: flags.ethereal(),
        is_runeword: flags.is_runeword(),
        durability_cur: pi.item.current_durability,
        durability_max: pi.item.max_durability,
        base_defense: base_def_opt.map(|(mn, _)| mn),
        max_defense: base_def_opt.map(|(_, mx)| mx),
        base_damage_1h_min: dmg_opt.map(|(mn, _, _, _)| mn).filter(|&v| v > 0),
        base_damage_1h_max: dmg_opt.map(|(_, mx, _, _)| mx).filter(|&v| v > 0),
        base_damage_2h_min: dmg_opt.map(|(_, _, hmn, _)| hmn).filter(|&v| v > 0),
        base_damage_2h_max: dmg_opt.map(|(_, _, _, hmx)| hmx).filter(|&v| v > 0),
        req_strength: if req_str > 0 { Some(req_str) } else { None },
        req_dexterity: if req_dex > 0 { Some(req_dex) } else { None },
        req_level: if req_lvl > 0 { Some(req_lvl) } else { None },
        skill_bonuses: {
            let mut v = Vec::new();
            for sl in &pi.item.stat_lists {
                v.extend(extract_skill_bonuses_with_opt(&sl.stats,
                    resolver_opt.map(|(c, r)| (c, r.as_ref()))));
            }
            v
        },
        // US-019: 复用行414 已计算的 stats_cat,消除重复 categorize_item_stats 调用
        stats: stats_cat_for_field,
        tooltip: Some(classified),
        socketed_items: pi.item.socketed_items.iter().map(|si| {
            // Build a simplified summary for each socketed item
            // US-018: 用缓存的 item_size
            let (sw, sh) = cache.item_size(&si.code);
            StoredItemSummary {
                code: si.code.clone(),
                x: si.x,
                y: si.y,
                amount: si.amount,
                inv_width: sw,
                inv_height: sh,
                quality: Some(si.quality.as_u8()).filter(|&q| (2..=8).contains(&q)),
                identified: si.flags.identified(),
                name_zh: None,
                name_en: None,
                name_zh_tw: None,
                page: si.page.map(|p| p.as_u8()),
                skill_bonuses: Vec::new(),
                raw_offset: None,
                raw_length: 0,
                item_level: if si.item_level > 0 { Some(si.item_level) } else { None },
                num_sockets: si.num_sockets,
                is_ethereal: si.flags.ethereal(),
                is_runeword: si.flags.is_runeword(),
                durability_cur: si.current_durability,
                durability_max: si.max_durability,
                base_defense: None,
                max_defense: None,
                base_damage_1h_min: None,
                base_damage_1h_max: None,
                base_damage_2h_min: None,
                base_damage_2h_max: None,
                req_strength: None,
                req_dexterity: None,
                req_level: None,
                socketed_items: Vec::new(),
                stats: ItemStats::default(),
                tooltip: None,
            }
        }).collect(),
    }
}

/// 角色信息响应（前端展示）。
#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterInfoResult {
    pub name: String,
    pub class: String,
    pub class_en: String,
    pub class_cn: String,
    pub class_zh_tw: String,
    pub level: u8,
    pub experience: u32,
    pub strength: u32,
    pub energy: u32,
    pub dexterity: u32,
    pub vitality: u32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_mana: u32,
    pub max_mana: u32,
    pub current_stamina: u32,
    pub max_stamina: u32,
    pub is_hardcore: bool,
    pub is_expansion: bool,
    /// 文件创建时间 (Unix timestamp)
    pub creation_time: u32,
    /// 文件原始字节大小
    pub file_size: u32,
    /// 文件 SHA-256 哈希
    pub file_hash: String,
    pub last_played: u32,
    /// 背包金币
    pub gold: u32,
    /// 仓库金币
    pub gold_bank: u32,
    /// 未分配属性点
    pub stat_points: u32,
    /// 未分配技能点
    pub new_skills: u32,
    /// d2s 文件源路径（便于前端展示）
    pub source_path: String,
    /// 装备位列表（与 EquipmentPanel SLOT_GRID_AREA 对齐）
    pub equipment: Vec<EquipmentSlotInfo>,
    /// 背包物品（JM 段 ItemMode::Stored 的 ParsedItem 精简摘要）
    #[serde(default)]
    pub backpack_items: Vec<StoredItemSummary>,
    /// 腰带物品（JM 段 ItemMode::Belt）
    #[serde(default)]
    pub belt_items: Vec<StoredItemSummary>,
    /// 个人仓库物品（d2s JM 段 Page=MyStash(5),16×16 网格）
    #[serde(default)]
    pub personal_stash_items: Vec<StoredItemSummary>,
    /// 已解码技能（from "if" 段）
    #[serde(default)]
    pub skills_decoded: Vec<SkillEntry>,
    /// 三难度小站标记
    #[serde(default)]
    pub waypoints: WaypointSet,
    /// Woo! 段任务数据 (含 progression + 原始 uint16 位掩码)
    #[serde(default)]
    pub woo: WooQuestData,
    /// w4 NPC 对话/奖励消费状态
    #[serde(default)]
    pub w4: W4DialogData,
    /// 任务进度 (从 woo 数据推导的摘要)
    #[serde(default)]
    pub quests: Vec<QuestEntry>,
    /// 魔改 layout 标记 (D2R 原版始终 false)
    #[serde(default)]
    pub is_modified_layout: bool,
    /// 已确认的二进制结构摘要（便于前端/调试展示）。
    pub binary_structure: CharacterBinaryStructure,
    /// 佣兵装备（helm / armor / weapon_main / shield_main）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merc_equipment: Vec<EquipmentSlotInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBinaryStructure {
    pub detected_layout: String,
    pub active_weapon: u32,
    pub attributes_offset: usize,
    pub protocol_equipped_slots: usize,
    pub display_equipped_slots: usize,
    pub item_layout: CharacterItemLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterItemLayout {
    pub location_id_bit_offset: u8,
    pub equipped_slot_bit_offset: u8,
    pub huffman_code_bit_offset: u8,
    pub socket_count_bits: u8,
    pub uid_bits: u8,
    pub ilvl_bits: u8,
    pub quality_bits: u8,
    pub stat_terminator: u16,
}

/// 物品 stat 分类结构。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemStats {
    /// 基础属性 (normal/superior 底材的固有增强属性)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base: Vec<ItemStat>,
    /// 词缀/魔法属性 (magic/rare/unique/crafted 的主词缀, set 单件属性, runeword 属性)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affix: Vec<ItemStat>,
    /// 符文之语额外词缀 (stat_lists[1..] for runeword items)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runeword: Vec<ItemStat>,
    /// 套装词缀 (set partial/full completion bonuses)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set_bonus: Vec<ItemStat>,
}

/// 按品质和符文标识，将 stat_lists 分配到对应分类。
pub fn categorize_item_stats(
    quality_byte: u8,
    is_runeword: bool,
    stat_lists: &[crate::protocol::common::StatList],
) -> ItemStats {
    let mut s = ItemStats::default();
    if is_runeword {
        // 符文之语: stat_lists[0] = 符文之语属性, [1..] = 额外符文词缀
        s.affix = stat_lists.first().map(|sl| sl.stats.clone()).unwrap_or_default();
        if stat_lists.len() > 1 {
            s.runeword = stat_lists[1..].iter().flat_map(|sl| sl.stats.clone()).collect();
        }
    } else {
        match quality_byte {
            1..=3 => {
                // Low/Normal/Superior: 多数是底材固有属性，
                // 但 mod 物品可能在 normal 品质上有技能词缀，需要区分
                if let Some(sl) = stat_lists.first() {
                    for st in &sl.stats {
                        if st.skill_id.is_some() || st.skill_tab.is_some() || st.max_charges.is_some() {
                            s.affix.push(st.clone());
                        } else {
                            s.base.push(st.clone());
                        }
                    }
                }
            }
            5 => {
                // 套装: stat_lists[0] = 单件属性, [1..] = 套装加成
                s.affix = stat_lists.first().map(|sl| sl.stats.clone()).unwrap_or_default();
                if stat_lists.len() > 1 {
                    s.set_bonus = stat_lists[1..].iter().flat_map(|sl| sl.stats.clone()).collect();
                }
            }
            _ => {
                // Magic/Rare/Unique/Crafted: 魔法词缀/暗金属性
                s.affix = stat_lists.first().map(|sl| sl.stats.clone()).unwrap_or_default();
            }
        }
    }
    s
}

/// 装备位信息（单槽）。
///
/// `occupied` 决定 EquipmentPanel 显示实色 border (装备) 还是 dashed (空槽)。
/// 当 `occupied=true` 时,`code/name_zh/name_en/quality` 才有意义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSlotInfo {
    pub slot: String,
    pub occupied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_zh_tw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    pub socketed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_bonuses: Vec<SkillBonus>,
    pub stats: ItemStats,
    #[serde(default)]
    pub durability_cur: u8,
    #[serde(default)]
    pub durability_max: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<TooltipData>,
}

// LocalizedNameMaps removed — use resource::NameResolver instead

/// 当前前后端统一使用的 12 槽布局。
///
/// 相比传统 8 槽,这里把主副武器栏拆开，并显式保留左右戒：
/// - `weapon_main` / `shield_main`: 当前武器组
/// - `weapon_alt` / `shield_alt`: 另一套武器组
pub const EQUIPMENT_SLOTS: [&str; 12] = [
    "helm",
    "amulet",
    "ring_l",
    "ring_r",
    "armor",
    "weapon_main",
    "shield_main",
    "weapon_alt",
    "shield_alt",
    "gloves",
    "boots",
    "belt",
];
#[cfg(test)]
fn empty_equipment_slots() -> Vec<EquipmentSlotInfo> {
    EQUIPMENT_SLOTS
        .iter()
        .map(|s| EquipmentSlotInfo {
            slot: s.to_string(),
            occupied: false,
            code: None,
            name_zh: None,
            name_en: None,
            name_zh_tw: None,
            quality: None,
            socketed: false,
            skill_bonuses: Vec::new(),
            durability_cur: 0,
            durability_max: 0,
            stats: ItemStats::default(),
            tooltip: None,
        })
        .collect()
}

/// 从 d2s 文件读取角色信息。
#[tauri::command]
pub fn read_character_info(state: State<AppState>, path: String) -> Result<CharacterInfoResult, String> {
    let (game_root, active_mod, profile_id, language) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.get_config("game_root").map_err(|e| e.to_string())?.unwrap_or_default(),
            db.get_config("active_mod").map_err(|e| e.to_string())?.unwrap_or_default(),
            crate::commands::config::get_active_profile_id(&db).unwrap_or(0),
            db.get_config("language").map_err(|e| e.to_string())?.unwrap_or_else(|| "zhCN".to_string()),
        )
    };
    let game_data_path = if game_root.is_empty() {
        String::new()
    } else {
        crate::commands::config::resolve_excel_path(&game_root, &active_mod)
    };
    // Build resolver using read-only db conn (avoids expensive Database::init + create_tables)
    // Build resolver with read-only db conn to avoid "database is locked" errors
    let resolver: Option<(rusqlite::Connection, Arc<NameResolver>)> = {
        let db_path = crate::database::Database::get_db_path_clone();
        if profile_id > 0 && db_path.exists() {
            rusqlite::Connection::open_with_flags(
                &db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ).ok()
            // US-022: 预热 localized_string 缓存,避免每次 resolve 走 1-4 次 SQLite 查询
            .map(|conn| {
                let resolver = crate::resource::get_cached_resolver(&conn, profile_id);
                (conn, resolver)
            })
        } else {
            None
        }
    };

    read_character_info_inner(
        &path,
        if game_data_path.is_empty() { None } else { Some(game_data_path.as_str()) },
        profile_id,
        &language,
        resolver,
    )
}
fn read_character_info_inner(
    path: &str,
    game_data_path: Option<&str>,
    _profile_id: i64,
    language: &str,
    resolver: Option<(rusqlite::Connection, Arc<NameResolver>)>,
) -> Result<CharacterInfoResult, String> {
    let _t0 = std::time::Instant::now();

    let data = std::fs::read(path).map_err(|e| format!("Failed to read d2s: {}", e))?;
    let file_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        hex::encode(hasher.finalize())
    };
    // P1: file_hash cache (切回同一角色 <100ms vs 解析 5-10s)
    if let Some(json) = read_char_cache_get(&file_hash)
        && let Ok(cached) = serde_json::from_str::<CharacterInfoResult>(&json) {
            log::info!("[TIMING] read_character_info: CACHE HIT {:?}", _t0.elapsed());
            return Ok(cached);
        }
    log::info!("[TIMING] read_character_info: 读文件 {:?}", _t0.elapsed());
    crate::data::stat_loader::set_runtime_excel_path(game_data_path);
    let _active_weapon = read_active_weapon(&data);
    log::info!("[TIMING] read_character_info: active_weapon {:?}", _t0.elapsed());
    // 标准 D2SLib layout (原版 D2R + D2RMM mod)
    let _t5 = std::time::Instant::now();
    let f = parse_d2s(&data).map_err(|e| format!("Failed to parse d2s: {}", e))?;
    log::info!("[TIMING] parse_d2s {:?} (eq={} bp={} belt={} cube={} merc={})", _t5.elapsed(),
        f.equipped.len(), f.backpack.len(), f.belt.len(), f.cube.len(), f.merc.len());
    let _t6 = std::time::Instant::now();

    // P0: 直接用 f.equipped(已分类),不再调 items::read_standard_items
    // 消除重复 d2s 扫描(原代码 5-10s 双重扫描)
    let standard_items: Vec<_> = f.equipped.iter()
        .filter(|pi| pi.item.mode == crate::protocol::common::ItemMode::Equipped)
        .cloned()
        .collect();
    log::info!("[TIMING] read_character_info: filter_equipped {:?} ({} items from parse_d2s)",
        _t0.elapsed(), standard_items.len());

    let equipment = if let Some((ref conn, ref resolver)) = resolver {
        crate::commands::character_equip::build_equipment_from_parsed_items(
            &standard_items, conn, resolver, language)
    } else {
        // resolver-less fallback: map items by slot with basic data, no tooltips
        let mut by_slot: std::collections::HashMap<&str, &crate::protocol::d2i::parser::ParsedItem> = std::collections::HashMap::new();
        for pi in &standard_items {
            if let Some(slot) = crate::commands::character_equip::location_to_slot(pi.item.location) {
                by_slot.entry(slot).or_insert(pi);
            }
        }
        EQUIPMENT_SLOTS.iter().map(|slot| {
            let pi = by_slot.get(slot).copied();
            EquipmentSlotInfo {
                slot: (*slot).to_string(),
                occupied: pi.is_some(),
                code: pi.map(|p| p.item.code.clone()),
                name_zh: None, name_en: None, name_zh_tw: None,
                quality: pi.and_then(|p| quality_key_from_byte(p.item.quality.as_u8())),
                socketed: pi.is_some_and(|p| p.item.flags.socketed()),
                skill_bonuses: pi.map_or(Vec::new(), |p| {
                    let mut bonuses = Vec::new();
                    for sl in &p.item.stat_lists {
                        bonuses.extend(extract_skill_bonuses(&sl.stats));
                    }
                    bonuses
                }),
                durability_cur: pi.map_or(0, |p| p.item.current_durability),
            stats: pi.map_or(ItemStats::default(), |p| {
                categorize_item_stats(p.item.quality.as_u8(), p.item.flags.is_runeword(), &p.item.stat_lists)
            }),
                durability_max: pi.map_or(0, |p| p.item.max_durability),
                tooltip: None,
            }
        }).collect()
    };
    log::info!("[TIMING] build_equipment {:?} ({} slots)", _t6.elapsed(), equipment.len());
    let _t7 = std::time::Instant::now();

    let result = character_to_result(
        &f,
        path,
        equipment,
        build_binary_structure("standard-v105", f.header.active_weapon),
        &file_hash,
        resolver.as_ref(),
        language,
    );
    log::info!("[TIMING] character_to_result {:?}", _t7.elapsed());
    log::info!("[TIMING] TOTAL read_character_info {:?}", _t0.elapsed());
    // P1: 写 cache
    if let Ok(json) = serde_json::to_string(&result) {
        read_char_cache_put(&file_hash, json);
    }
    Ok(result)
}

// ── P1: file_hash cache (JSON 序列化) ────────────────────────────────
static CHAR_CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::OnceLock::new();
const CHAR_CACHE_CAP: usize = 64;

fn char_cache() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    CHAR_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::with_capacity(CHAR_CACHE_CAP)))
}

fn read_char_cache_get(file_hash: &str) -> Option<String> {
    char_cache().lock().ok()?.get(file_hash).cloned()
}

fn read_char_cache_put(file_hash: &str, json: String) {
    let Ok(mut cache) = char_cache().lock() else { return };
    if cache.len() >= CHAR_CACHE_CAP
        && let Some(k) = cache.keys().next().cloned() {
            cache.remove(&k);
        }
    cache.insert(file_hash.to_string(), json);
}



pub fn quality_key_from_byte(quality_byte: u8) -> Option<String> {
    match quality_byte {
        1 => Some("low".to_string()),
        2 => Some("normal".to_string()),
        3 => Some("superior".to_string()),
        4 => Some("magic".to_string()),
        5 => Some("set".to_string()),
        6 => Some("rare".to_string()),
        7 => Some("unique".to_string()),
        8 => Some("crafted".to_string()),
        _ => None,
    }
}

pub fn build_equipment_tooltip_lines(
    slot: &str,
    code: &str,
    quality_byte: u8,
    item_level: Option<u8>,
    title_en: Option<&str>,
    title_zh: Option<&str>,
    runeword_id: Option<u16>,
    unique_id: Option<u16>,
    set_id: Option<u16>,
    main_stats: &[crate::protocol::common::ItemStat],
    runeword_stats: &[crate::protocol::common::ItemStat],
    language: &str,
    conn: Option<&rusqlite::Connection>,
    profile_id: i64,
) -> Vec<String> {
    crate::resource::TooltipFormatter::equipment_tooltip(
        slot, code, quality_byte, item_level,
        title_en, title_zh, runeword_id, unique_id, set_id,
        main_stats, runeword_stats,
        0, 0, 0, language, conn, profile_id,
        true,
    )
}


fn read_active_weapon(data: &[u8]) -> u32 {
    if data.len() >= 0x14 {
        u32::from_le_bytes([data[0x10], data[0x11], data[0x12], data[0x13]])
    } else {
        0
    }
}

/// 从 Woo! quest uint16 位掩码推导 `Vec<QuestEntry>`。
/// 
/// CLI QUEST_DATA_INDICES 映射 display 位置 → 1-indexed data 位置。
/// quest_id 为 display 顺序 (0-based)，数据从对应的 data_idx 读。
fn woo_quests_to_entries(woo: &WooQuestData) -> Vec<QuestEntry> {
    // CLI QUEST_DATA_INDICES: display 位置 → 1-indexed data 位置
    let data_indices: [&[usize]; 5] = [
        &[1, 2, 3, 5, 4, 6],  // ActI
        &[1, 2, 4, 5, 6, 7],  // ActII
        &[4, 3, 1, 2, 5, 6],  // ActIII
        &[1, 2, 3],           // ActIV
        &[3, 4, 5, 6, 8, 7],  // ActV
    ];
    let qf_standard: u16 = (1 << 0) | (1 << 8) | (1 << 9) | (1 << 13) | (1 << 14) | (1 << 15);
    let mut entries = Vec::new();
    let _act_quest_counts: [u8; 5] = [8, 8, 8, 8, 16];
    for (diff, acts) in woo.difficulties.iter().enumerate() {
        for (ai, quests) in acts.iter().enumerate() {
            let indices = data_indices.get(ai).copied().unwrap_or(&[]);
            for (qi, &data_idx) in indices.iter().enumerate() {
                let mask = if data_idx < quests.len() { quests[data_idx] } else { 0 };
                // 交易的工具 (Act 1, data_idx=5): bit 0 = 奖励可用(打孔/注入)
                // 不等同于完成，需要 bit 8/9/14/15 才算真正完成
                let completed = if ai == 0 && data_idx == 5 {
                    mask & ((1 << 8) | (1 << 9) | (1 << 14) | (1 << 15)) != 0
                } else {
                    let has_standard = mask & qf_standard != 0;
                    let has_multi = mask.count_ones() >= 3;
                    has_standard || has_multi
                };
                entries.push(QuestEntry {
                    difficulty: diff as u8,
                    act: (ai + 1) as u8,
                    quest_id: qi as u8,
                    completed,
                });
            }
        }
    }
    entries
}

fn build_binary_structure(detected_layout: &str, active_weapon: u32) -> CharacterBinaryStructure {
    let item_layout = known_item_bit_layout();
    CharacterBinaryStructure {
        detected_layout: detected_layout.to_string(),
        active_weapon,
        attributes_offset: ATTRIBUTES_OFFSET,
        protocol_equipped_slots: 12,
        display_equipped_slots: EQUIPMENT_SLOTS.len(),
        item_layout: CharacterItemLayout {
            location_id_bit_offset: item_layout.location_id_bit_offset,
            equipped_slot_bit_offset: item_layout.equipped_slot_bit_offset,
            huffman_code_bit_offset: item_layout.huffman_code_bit_offset,
            socket_count_bits: item_layout.socket_count_bits,
            uid_bits: item_layout.uid_bits,
            ilvl_bits: item_layout.ilvl_bits,
            quality_bits: item_layout.quality_bits,
            stat_terminator: item_layout.stat_terminator,
        },
    }
}

/// 构造魔改 layout 的 CharacterInfoResult。
///
/// 魔改 layout attributes 全是噪声 — Level/Class 从魔改 header 字段读取,
/// 其他 attributes 全部填 0,前端按 `is_modified_layout=true` 提示用户。
/// 把 d2s 解析结果转成前端友好的响应。
///
pub fn character_to_result(
    f: &D2SCharacter,
    source_path: &str,
    equipment: Vec<EquipmentSlotInfo>,
    binary_structure: CharacterBinaryStructure,
    file_hash: &str,
    resolver_opt: Option<&(Connection, Arc<NameResolver>)>,
    language: &str,
) -> CharacterInfoResult {
    let class = f.header.character_class();
    use crate::protocol::d2s::attributes::AttributeId;
    let attrs = &f.attributes;

    let mut name_cache: std::collections::HashMap<(String, u8, u16, u16, String), String> = std::collections::HashMap::new();
    // US-018: 预热物品基础数据查找缓存,O(n) 一次性扫描 → 之后 O(1) lookup
    let mut item_cache = ItemLookupCache::new();
    item_cache.warmup();

    let t_item = std::time::Instant::now();
    let backpack_items: Vec<_> = f.backpack.iter().chain(f.cube.iter()).map(|pi| {
        let t = std::time::Instant::now();
        let summary = build_stored_item_summary(pi, resolver_opt, &mut name_cache, &mut item_cache, language);
        let elapsed = t.elapsed();
        // US-024: 单物品 > 5ms 报警
        if elapsed.as_millis() > 5 {
            log::warn!("[c2r] SLOW item: code={} elapsed={}ms", pi.item.code, elapsed.as_millis());
        }
        summary
    }).collect();
    log::info!("[c2r] backpack({})+cube({}) done: {} ms", f.backpack.len(), f.cube.len(), t_item.elapsed().as_millis());

    let t_belt = std::time::Instant::now();
    let mut belt_items: Vec<_> = f.belt.iter().map(|pi| build_stored_item_summary(pi, resolver_opt, &mut name_cache, &mut item_cache, language)).collect();
    belt_items.sort_by_key(|b| (b.y, b.x));
    log::info!("[c2r] belt({}) done: {} ms", f.belt.len(), t_belt.elapsed().as_millis());

    let t_stash = std::time::Instant::now();
    let mut personal_stash_items: Vec<_> = f.personal_stash.iter().map(|pi| build_stored_item_summary(pi, resolver_opt, &mut name_cache, &mut item_cache, language)).collect();
    personal_stash_items.sort_by_key(|b| (b.y, b.x));
    log::info!("[c2r] personal_stash({}) done: {} ms", f.personal_stash.len(), t_stash.elapsed().as_millis());

    let t_merc = std::time::Instant::now();
    let merc_equip = build_merc_equipment(&f.merc, resolver_opt, language);
    log::info!("[c2r] merc({}) done: {} ms", f.merc.len(), t_merc.elapsed().as_millis());

    log::info!("[c2r] 构建 CharacterInfoResult...");

    CharacterInfoResult {
        name: f.header.name.clone(),
        class: format!("{:?}", class),
        class_en: class.name_en().to_string(),
        class_cn: class.name_cn().to_string(),
        class_zh_tw: class.name_tw().to_string(),
        level: attrs.get(AttributeId::Level) as u8,
        experience: attrs.get(AttributeId::Experience),
        strength: attrs.get(AttributeId::Strength),
        energy: attrs.get(AttributeId::Energy),
        dexterity: attrs.get(AttributeId::Dexterity),
        vitality: attrs.get(AttributeId::Vitality),
        current_hp: attrs.get(AttributeId::Hitpoints),
        max_hp: attrs.get(AttributeId::MaxHp),
        current_mana: attrs.get(AttributeId::Mana),
        max_mana: attrs.get(AttributeId::MaxMana),
        current_stamina: attrs.get(AttributeId::Stamina),
        max_stamina: attrs.get(AttributeId::MaxStamina),
        creation_time: f.header.save_timestamp,
        file_size: f.header.filesize,
        file_hash: file_hash.to_string(),
        is_hardcore: f.header.is_hardcore(),
        is_expansion: f.header.is_expansion(),
        last_played: f.header.last_played(),
        gold: attrs.get(AttributeId::Gold),
        gold_bank: attrs.get(AttributeId::GoldBank),
        stat_points: attrs.get(AttributeId::StatPoints),
        new_skills: attrs.get(AttributeId::NewSkills),
        source_path: source_path.to_string(),
        equipment,
        backpack_items,
        belt_items,
        personal_stash_items,
        skills_decoded: f.skills_decoded.clone(),
        waypoints: f.waypoints.clone(),
        woo: f.woo.clone(),
        w4: f.w4.clone(),
        quests: woo_quests_to_entries(&f.woo),
        is_modified_layout: false,
        binary_structure,
        merc_equipment: merc_equip,
    }
}


/// 后台分阶段加载角色，通过 Tauri Event 分片推送到前端。
/// - stage1: 头信息（名字/职业/等级）→ 立即显示
/// - stage2: 装备数据 → 渲染装备栏
/// - stage3: 全量数据 → 完成加载
/// 前端监听 `char:stage1` / `char:stage2` / `char:stage3` / `char:error` 事件。
#[tauri::command]
pub async fn load_character_background(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    log::info!("[load_character_background] 入口: path={}", path);
    let (game_root, active_mod, profile_id, language) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.get_config("game_root").map_err(|e| e.to_string())?.unwrap_or_default(),
            db.get_config("active_mod").map_err(|e| e.to_string())?.unwrap_or_default(),
            crate::commands::config::get_active_profile_id(&db).unwrap_or(0),
            db.get_config("language").map_err(|e| e.to_string())?.unwrap_or_else(|| "zhCN".to_string()),
        )
    };
    log::info!("[load_character_background] config: game_root='{}' mod='{}' profile_id={} language='{}'",
        game_root, active_mod, profile_id, language);

    let game_data_path = if game_root.is_empty() {
        String::new()
    } else {
        crate::commands::config::resolve_excel_path(&game_root, &active_mod)
    };
    log::info!("[load_character_background] game_data_path='{}'", game_data_path);

    let app_clone = app.clone();
    std::thread::spawn(move || {
        log::info!("[load_character_background] 线程启动");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match load_character_stages(&path, &game_data_path, profile_id, &language, &app_clone) {
                Ok(()) => {
                    log::info!("[load_character_background] 线程完成 OK");
                }
                Err(e) => {
                    log::error!("[load_character_background] 线程失败: {}", e);
                    let _ = app_clone.emit("char:error", &e);
                }
            }
        }));
        if let Err(panic_err) = result {
            let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "加载角色线程崩溃(未知原因)".to_string()
            };
            log::error!("[load_character_background] 线程崩溃: {}", msg);
            let _ = app_clone.emit("char:error", &msg);
        }
    });
    Ok(())
}

fn load_character_stages(
    path: &str,
    game_data_path: &str,
    profile_id: i64,
    language: &str,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    use crate::protocol::common::ItemMode;
    log::info!("[stages] start: path={} gdp={} pid={} lang={}", path, game_data_path, profile_id, language);

    let gdp_opt = if game_data_path.is_empty() { None } else { Some(game_data_path) };

    // ── Read file + hash ──
    let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let file_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        hex::encode(hasher.finalize())
    };
    let file_size = data.len() as u32;
    log::info!("[stages] 读文件 OK: {} bytes, hash={}", file_size, &file_hash[..12]);

    // ── Stage 1: Quick header ──
    {
        use crate::protocol::d2s::header::D2SHeader;
        let header = D2SHeader::from_bytes(&data)
            .map_err(|e| format!("Header 解析失败: {}", e))?;
        let cc = header.character_class();
        let level_byte = if data.len() > 0x1C { data[0x1B] } else { 0 };

        let _ = app.emit("char:stage1", serde_json::json!({
            "name": header.name,
            "class": format!("{:?}", cc),
            "class_en": cc.name_en(),
            "class_cn": cc.name_cn(),
            "class_zh_tw": cc.name_tw(),
            "level": level_byte,
            "is_hardcore": header.is_hardcore(),
            "is_expansion": header.is_expansion(),
            "file_size": file_size,
            "file_hash": file_hash,
            "last_played": header.last_played(),
            "source_path": path,
        }));
        log::info!("[stages] stage1 emitted: name={:?} class={:?} level={}",
            header.name, cc, level_byte);
    }
    crate::data::stat_loader::set_runtime_excel_path(gdp_opt);

    // ── Stage 2: Equipment ──
    // US-018: 预热物品基础数据缓存,用于加速后续 armor_stats/weapon_stats/item_size 查找
    let mut item_cache = ItemLookupCache::new();
    item_cache.warmup();
    let resolver: Option<(rusqlite::Connection, Arc<NameResolver>)> = {
        let db_path = crate::database::Database::get_db_path_clone();
        if profile_id > 0 && db_path.exists() {
            rusqlite::Connection::open_with_flags(
                &db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ).ok()
            // US-022: 预热 localized_string 缓存,避免每次 resolve 走 1-4 次 SQLite 查询
            .map(|conn| {
                let resolver = crate::resource::get_cached_resolver(&conn, profile_id);
                (conn, resolver)
            })
        } else {
            None
        }
    };
    log::info!("[stages] resolver: {}", if resolver.is_some() { "Some(conn)" } else { "None" });

    // P0 fix: parse_d2s 一次(线程后期还要用),直接取 f.equipped
    // 避免 items::read_standard_items 双重扫描(5-10s 慢路径)
    let t_parse_start = std::time::Instant::now();
    log::info!("[stages] parse_d2s 起始 (size={} bytes)", data.len());
    let f = parse_d2s(&data).map_err(|e| format!("角色解析失败: {}", e))?;
    log::info!("[stages] parse_d2s 完成: {} ms (eq={} bp={} belt={} cube={} merc={})",
        t_parse_start.elapsed().as_millis(),
        f.equipped.len(), f.backpack.len(), f.belt.len(), f.cube.len(), f.merc.len());
    if t_parse_start.elapsed().as_secs() > 5 {
        log::warn!("[stages] parse_d2s 慢({}s),考虑 mod 物品异常", t_parse_start.elapsed().as_secs());
    }

    // 提取装备(code + location),用于 resolver-less 占位 + 后续补全
    let standard_items: Vec<_> = f.equipped.iter()
        .filter(|pi| pi.item.mode == ItemMode::Equipped)
        .cloned()
        .collect();
    log::info!("[stages] standard_items: {} 件", standard_items.len());

    let mut by_slot: std::collections::HashMap<&str, &crate::protocol::d2i::parser::ParsedItem> = std::collections::HashMap::new();
    for pi in &standard_items {
        if let Some(slot) = crate::commands::character_equip::location_to_slot(pi.item.location) {
            by_slot.entry(slot).or_insert(pi);
        }
    }
    let equipment: Vec<EquipmentSlotInfo> = EQUIPMENT_SLOTS.iter().map(|slot| {
    let pi = by_slot.get(slot).copied();
    // Resolve names once (shared by struct field + tooltip)
    let (mut name_zh_opt, mut name_en_opt) = match (pi, resolver.as_ref()) {
        (Some(p), Some((conn, r))) => {
            let zh = r.resolve(conn, &p.item.code, Some(p.item.quality.as_u8()), p.item.unique_id, p.item.set_id, "zhCN").display_name;
            let en = r.resolve(conn, &p.item.code, Some(p.item.quality.as_u8()), p.item.unique_id, p.item.set_id, "enUS").display_name;
            (Some(zh), Some(en))
        }
        _ => (None, None),
    };
    // 符文之语名称追加 [隐秘]
    if let Some(p) = pi
        && p.item.flags.is_runeword() && !p.item.socketed_items.is_empty() {
            let rc: Vec<&str> = p.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
            if let Some(rw_en) = crate::data::runewords::match_runeword(&rc) {
                name_en_opt = Some(format!("{} [{}]", name_en_opt.as_deref().unwrap_or("?"), rw_en));
                if let Some(rw_zh) = crate::data::runewords::match_runeword_zh(&rc) {
                    name_zh_opt = Some(format!("{}[{}]", name_zh_opt.as_deref().unwrap_or("?"), rw_zh));
                }
            }
        }
    // Build structured tooltip for equipment (使用分组字段)
    let tooltip_data = match (pi, resolver.as_ref()) {
        (Some(p), Some((conn, _))) => {
            let stats_cat = categorize_item_stats(p.item.quality.as_u8(), p.item.flags.is_runeword(), &p.item.stat_lists);
            let mut td = build_tooltip_from_stats(&stats_cat, language, Some(conn), profile_id);
            // 底材基础属性
            // 实际防御优先: item body 解析出的 defense (已含 ED 加成);0 时回退底材区间
            if let Some(def) = crate::data::items_base::armor_stats(&p.item.code) {
                if p.item.defense > 0 {
                    td.base_stats.insert(0, format!("防御: {}", p.item.defense));
                } else if def.minac > 0 || def.maxac > 0 {
                    td.base_stats.insert(0, format!("防御: {}-{}", def.minac, def.maxac));
                }
                if def.levelreq > 0 { td.base_stats.push(format!("需要等级: {}", def.levelreq)); }
                if def.reqstr > 0 { td.base_stats.push(format!("需要力量: {}", def.reqstr)); }
                if def.reqdex > 0 { td.base_stats.push(format!("需要敏捷: {}", def.reqdex)); }
            }
            // 盾牌 smite 伤害 (含圣骑士盾)
            if let Some((smin, smax)) = crate::data::items_base::shield_smite(&p.item.code) {
                td.base_stats.push(if smin == smax {
                    format!("盾击伤害: {}", smin)
                } else {
                    format!("盾击伤害: {}-{}", smin, smax)
                });
            }
            if let Some(wpn) = crate::data::items_base::weapon_stats(&p.item.code) {
                let (dmin, dmax) = wpn.display_damage();
                if dmin > 0 || dmax > 0 {
                    td.base_stats.push(if dmin == dmax {
                        format!("攻击力: {}", dmin)
                    } else {
                        format!("攻击力: {}-{}", dmin, dmax)
                    });
                }
                if wpn.levelreq > 0 { td.base_stats.push(format!("需要等级: {}", wpn.levelreq)); }
                if wpn.reqstr > 0 { td.base_stats.push(format!("需要力量: {}", wpn.reqstr)); }
                if wpn.reqdex > 0 { td.base_stats.push(format!("需要敏捷: {}", wpn.reqdex)); }
            }
            if p.item.max_durability > 0 {
                td.base_stats.push(format!("耐久度: {}/{}", p.item.current_durability, p.item.max_durability));
            }
            if p.item.item_level > 0 {
                td.hidden_info.push(format!("物品等级: {}", p.item.item_level));
            }
            // 符文之语等级需求
            if p.item.flags.is_runeword() && !p.item.socketed_items.is_empty() {
                let rune_codes: Vec<String> = p.item.socketed_items.iter().map(|si| si.code.clone()).collect();
                let rw_level = crate::data::runewords::runeword_req_level(&rune_codes);
                if rw_level > 0 {
                    td.base_stats.push(format!("需要等级: {}", rw_level));
                }
            }
            // 魔法词缀等级需求
            if p.item.quality.as_u8() == 4 && (p.magic_prefix_id.is_some() || p.magic_suffix_id.is_some()) {
                let req = crate::resource::queries::get_magic_item_req_level(
                    conn, profile_id, p.magic_prefix_id, p.magic_suffix_id);
                if req > 0 {
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", req));
                }
            }
            // 暗金物品等级需求 (unique_item_def.level_req)
            if p.item.quality.as_u8() == 7
                && let Some(uid) = p.item.unique_id
                && let Some(def) = crate::resource::queries::get_unique_def(conn, profile_id, uid)
                && def.level_req > 0 {
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", def.level_req));
                }
            // 套装物品等级需求 (set_item_def.level_req)
            if p.item.quality.as_u8() == 5
                && let Some(sid) = p.item.set_id
                && let Some(def) = crate::resource::queries::get_set_item_by_item_id(conn, profile_id, sid)
                && def.level_req > 0 {
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", def.level_req));
                }
            // 套装加成 (绿色 set_bonus_stats, 来自 sets.txt)
            if p.item.quality.as_u8() == 5
                && let Some(sid) = p.item.set_id {
                    append_set_bonuses(&mut td, conn, profile_id, sid, language);
                }
            // 孔位 + 镶嵌
            if p.item.num_sockets > 0 {
                use crate::commands::character::{SocketedItemInfo, SocketsInfo};
                let socketed: Vec<SocketedItemInfo> = p.item.socketed_items.iter().map(|si| {
                    SocketedItemInfo {
                        code: si.code.clone(),
                        name_zh: None,
                        name_en: None,
                        quality: Some(si.quality.as_u8()).filter(|&q| (2..=8).contains(&q)),
                        amount: si.amount,
                    }
                }).collect();
                td.sockets = Some(SocketsInfo { count: p.item.num_sockets, items: socketed });
            }
            Some(td)
        }
        _ => None,
    };
    EquipmentSlotInfo {
        slot: (*slot).to_string(),
        occupied: pi.is_some(),
        code: pi.map(|p| p.item.code.clone()),
        name_zh: name_zh_opt,
        name_en: name_en_opt,
        name_zh_tw: None,
        quality: pi.and_then(|p| quality_key_from_byte(p.item.quality.as_u8())),
        socketed: pi.is_some_and(|p| p.item.flags.socketed()),
        skill_bonuses: pi.map_or(Vec::new(), |p| {
            let rp: Option<(&rusqlite::Connection, &crate::resource::NameResolver)> =
                resolver.as_ref().map(|(c, r)| (c, r.as_ref()));
            let mut bonuses = Vec::new();
            for sl in &p.item.stat_lists {
                bonuses.extend(extract_skill_bonuses_with_opt(&sl.stats, rp));
            }
            bonuses
        }),
        durability_cur: pi.map_or(0, |p| p.item.current_durability),
        durability_max: pi.map_or(0, |p| p.item.max_durability),
        stats: pi.map_or(ItemStats::default(), |p| {
            categorize_item_stats(p.item.quality.as_u8(), p.item.flags.is_runeword(), &p.item.stat_lists)
        }),
        tooltip: tooltip_data,
    }
    }).collect();
    log::info!("[load_character_stages] stage2 equipment JSON: {}",
        serde_json::to_string(&equipment).unwrap_or_else(|e| format!("<serialize error: {}>", e)));
    let _ = app.emit("char:stage2", serde_json::json!({
        "equipment": equipment,
    }));
    log::info!("[stages] stage2 emitted: {} slots", equipment.len());

    // ── Stage 3: Full parse ──
    // 复用 Stage 2 的 parse_d2s 结果,避免重复扫描
    // resolver 启用时 mod 物品的 stat 查询会导致线程 hang (待排查)
    log::info!("[stages] stage3 起始: character_to_result (resolver=skip)...");
    let t_t3_start = std::time::Instant::now();
    let active_weapon = read_active_weapon(&data);

    let character = character_to_result(
        &f,
        path,
        equipment,
        build_binary_structure("standard-v105", active_weapon),
        &file_hash,
        resolver.as_ref(),
        language,
    );
    log::info!("[stages] character_to_result 完成: {} ms", t_t3_start.elapsed().as_millis());

    log::info!("[load_character_stages] stage3 full data size: {} slots, {} backpack, {} merc, {} belt, {} personal_stash",
        character.equipment.len(), character.backpack_items.len(), character.merc_equipment.len(),
        character.belt_items.len(), character.personal_stash_items.len());
    log::info!("[stages] stage3 emit 开始...");
    // 测量 payload 序列化大小
    let payload_json = serde_json::to_string(&character).unwrap_or_else(|e| format!("<serialize error: {}>", e));
    log::info!("[stages] stage3 payload JSON 大小: {} 字节", payload_json.len());
    if payload_json.len() > 1_000_000 {
        log::warn!("[stages] stage3 payload >1MB, 可能影响 Tauri IPC 性能");
    }
    if let Err(e) = app.emit("char:stage3", &character) {
        log::error!("[stages] stage3 emit 失败: {:?}", e);
    }
    log::info!("[stages] stage3 emit 完成");
    Ok(())
}

/// 获取本地化技能文本（名称、描述、属性标签等）。
/// 返回 { string_key → text_value } 映射，供前端 tooltip 渲染用。
#[tauri::command]
pub fn get_localized_skill_texts(state: State<AppState>, language: String) -> Result<std::collections::HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.get_connection();
    let profile_id = crate::commands::config::get_active_profile_id(&db).unwrap_or(0);

    let mut map = std::collections::HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT string_key, text_value FROM localized_string
         WHERE namespace = 'skills' AND language = ?1 AND profile_id = ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(rusqlite::params![language, profile_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?;

    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        map.insert(key, value);
    }
    Ok(map)
}

/// 列出目录中所有 .d2s 文件（不含扩展名的角色名）。
#[tauri::command]
pub fn list_characters(dir: String) -> Result<Vec<String>, String> {
    let path = Path::new(&dir);
    if !path.is_dir() {
        return Ok(Vec::new()); // 目录不存在 → 空列表
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if let Some(ext) = p.extension()
            && ext == "d2s"
                && let Some(stem) = p.file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
    }
    names.sort();
    Ok(names)
}
/// Extract all items from a character .d2s into the extended warehouse.
/// Does NOT modify the .d2s file — items remain in-game.
#[tauri::command]
pub fn extract_character_equipment(
    state: State<AppState>,
    path: String,
    include_backpack: Option<bool>,
    include_equipped: Option<bool>,
) -> Result<crate::commands::stash::ExtractResult, String> {
    let include_backpack = include_backpack.unwrap_or(true);
    let include_equipped = include_equipped.unwrap_or(true);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    extract_character_equipment_inner(&db, path, include_backpack, include_equipped)
}

/// Core extraction logic (no tauri state) — testable via CLI/tests.
pub fn extract_character_equipment_inner(
    db: &crate::database::Database,
    path: String,
    include_backpack: bool,
    include_equipped: bool,
) -> Result<crate::commands::stash::ExtractResult, String> {
    use crate::protocol::common::{ItemMode, ItemLocation, ItemQuality};
    use crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP;
    use crate::database::models::WarehousedItem;

    // 1. Read and parse the .d2s
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read d2s: {}", e))?;
    let file = crate::protocol::d2s::parser::parse_file(&data)
        .map_err(|e| format!("Failed to parse d2s: {}", e))?;

    // US-018: 预热 item_size/armor_stats/weapon_stats 缓存(独立于 character_to_result)
    let mut item_cache = ItemLookupCache::new();
    item_cache.warmup();
    let _ = &item_cache; // 当前 extract_character_equipment 走独立路径,这里仅预热作为后续优化基线

    let source_char = file.header.name.clone();
    let save_path = path.clone();

    let class_cn = file.header.character_class().name_cn();
    let page_name = format!("{}的装备·{}", source_char, class_cn);
    // 2a. 找到 .d2s 文件中 JM 段偏移(用于提取原始比特)
    let marker = crate::protocol::d2s::parser::marker_offsets(&data);
    let d2s_jm_offset = marker.first_jm;
    // 2. Build WarehousedItems
    let active_profile_key = crate::commands::config::get_active_profile_key(db).unwrap_or_default();
    let mut warehouse_ids: Vec<String> = Vec::new();
    let mut equipped_count = 0usize;
    let mut backpack_count = 0usize;
    let mut belt_count = 0usize;
    let mut skipped: Vec<crate::commands::stash::SkippedItemReason> = Vec::new();

    let loc_to_slot = |loc: &ItemLocation| -> Option<&'static str> {
        match loc {
            ItemLocation::Head => Some("helm"),
            ItemLocation::Neck => Some("amulet"),
            ItemLocation::Torso => Some("armor"),
            ItemLocation::RightHand => Some("weapon_main"),
            ItemLocation::LeftHand => Some("shield_main"),
            ItemLocation::RightFinger => Some("ring_l"),
            ItemLocation::LeftFinger => Some("ring_r"),
            ItemLocation::Waist => Some("belt"),
            ItemLocation::Feet => Some("boots"),
            ItemLocation::Hands => Some("gloves"),
            _ => None,
        }
    };

    let q_str = |q: &ItemQuality| -> Option<&'static str> {
        match q {
            ItemQuality::Unique => Some("unique"),
            ItemQuality::Set => Some("set"),
            ItemQuality::Rare => Some("rare"),
            ItemQuality::Magic => Some("magic"),
            ItemQuality::Superior => Some("superior"),
            ItemQuality::Low => Some("low"),
            ItemQuality::Normal => Some("normal"),
            _ => None,
        }
    };

    // 每件物品携带其所属 JM 段的 payload 起点：
    // 角色物品段（equipped/belt/backpack/cube）→ first_jm+4；
    // 雇佣兵段（merc）→ merc_jm+4。提取 raw bits 必须用对应段的起点，
    // 否则 merc 装备会从角色物品段错位提取（混入背包/药水数据）。
    let first_payload = d2s_jm_offset.map(|o| o + 4).unwrap_or(0);
    let merc_payload = crate::protocol::d2s::parser::marker_offsets(&data)
        .merc_jm
        .map(|o| o + 4)
        .unwrap_or(first_payload);
    let all_pi: Vec<(&crate::protocol::d2i::parser::ParsedItem, usize)> = file
        .equipped
        .iter()
        .map(|p| (p, first_payload))
        .chain(file.belt.iter().map(|p| (p, first_payload)))
        .chain(file.backpack.iter().map(|p| (p, first_payload)))
        .chain(file.cube.iter().map(|p| (p, first_payload)))
        .chain(file.merc.iter().map(|p| (p, merc_payload)))
        .collect();
    // 过滤掉消耗品/工具类物品: 药水、卷轴、书、盒子
    let skip_codes: std::collections::HashSet<&str> = [
        "hp1","hp2","hp3","hp4","hp5","hpf","hpo",
        "mp1","mp2","mp3","mp4","mp5","mpf","mpo",
        "rvs","rvl","rps",
        "vps","wms","yps","bpl","bps","elx",
        "tsc","isc",
        "tbk","ibk",
        "box",
    ].into_iter().collect();

    for (pi, payload_start) in &all_pi {
        let code = &pi.item.code;
        let is_equipped = pi.item.mode == ItemMode::Equipped;
        let is_stored = pi.item.mode == ItemMode::Stored || pi.item.mode == ItemMode::Belt;

        if is_equipped && !include_equipped { continue; }
        if is_stored && !include_backpack { continue; }
        if pi.item.mode == ItemMode::Socket { continue; }
        if skip_codes.contains(code.as_str()) { continue; }

        let kind = ITEM_CODE_MAP.iter()
            .find(|(c, _, _, _)| *c == code.as_str())
            .map(|(_, _, k, _)| k.to_string())
            .unwrap_or_else(|| "misc".to_string());

        let slot = if is_equipped { loc_to_slot(&pi.item.location) } else { None };

        let wi = WarehousedItem {
            // id 必须全局唯一: {角色}-{code}-{本次序号}-{毫秒时间戳}。
            // 之前只用 {角色}-{code}-{序号},两次"存入仓库"序号相同会撞 PRIMARY KEY,
            // INSERT 失败被静默跳过 (表现为物品存不进去)。
            id: format!("{}-{}-{}-{}", source_char, code, warehouse_ids.len(), chrono::Utc::now().timestamp_millis()),
            item_code: code.clone(),
            item_name: String::new(),
            item_kind: kind,
            quality: q_str(&pi.item.quality).map(|s| s.to_string()),
            simple_item: pi.item.flags.simple_item(),
            quantity: pi.item.amount.max(1),
            profile_key: active_profile_key.clone(),
            game_version: String::new(),
            mod_name: String::new(),
            // 从 .d2s 提取原始 JM 比特（所有非简单物品——装备/背包/仓库都走
            // 标准解析器提取，数据完整；此前仅 equipped 走提取，背包物品会落入
            // encode_item_with_sockets 重编码，stat 被清空导致属性丢失）
            // 父物品 = 原始比特(保持 stat/quality 正确)
            // 镶嵌子物品 = 编码追加(mode=Socket)
            raw_item_bits: if !pi.item.flags.simple_item() && pi.raw_bit_length > 0 {
                let payload_start = *payload_start;
                let start_bit = pi.raw_bit_offset;
                let start_byte = (payload_start + start_bit / 8) as usize;
                // 只提取父物品的原始比特(含 item body + stat,不含子物品)
                let parent_bits = {
                    // 粗略估计:非紧凑体 ~400-600 bits,取 min(父长度, 400b=50B)
                    let parent_len_bits = pi.raw_bit_length;
                    let end_byte = (payload_start + (start_bit + parent_len_bits + 7) / 8) as usize;
                    if end_byte <= data.len() && start_byte < end_byte {
                        data[start_byte..end_byte].to_vec()
                    } else {
                        Vec::new()
                    }
                };
                if parent_bits.is_empty() || pi.item.socketed_items.is_empty() {
                    parent_bits
                } else {
                    // 扩大提取范围：覆盖 socket children。
                    // ★ 不能用 next_real（物品段最后一件时为 None，会一直提取到
                    //   文件末尾、把 merc 雇佣兵段混入 raw）。改为用同一 d2s 解析器
                    //   重扫物品段 flat，取父物品 + 后续连续 Socket children 的精确结束位。
                    // ★ 不能用 d2i parse_jm_page: 该解析器对 mod 物品位流游走与 d2s
                    //   items 解析器不一致（实测 lfw 被误解析成 spt、从 flat 丢失），
                    //   导致 cur=None 回退 parent_bits、镶嵌符文全部丢失。
                    //   同一解析器输出坐标必然与 pi.raw_bit_offset 同系。
                    let flat = if payload_start == merc_payload {
                        crate::protocol::d2s::items::read_merc_items(&data)
                    } else {
                        crate::protocol::d2s::items::read_standard_items(&data).unwrap_or_default()
                    };
                    let cur = flat.iter()
                        .filter(|f| f.item.code == pi.item.code)
                        .min_by_key(|f| (f.raw_bit_offset as i64 - pi.raw_bit_offset as i64).unsigned_abs());
                    let extended_end_bit = match cur {
                        Some(c) => {
                            let mut end = c.raw_bit_offset + c.raw_bit_length;
                            let mut cursor = end;
                            loop {
                                let next = flat.iter()
                                    .filter(|f| f.raw_bit_offset >= cursor)
                                    .min_by_key(|f| f.raw_bit_offset);
                                match next {
                                    Some(f) if f.item.mode == crate::protocol::common::ItemMode::Socket => {
                                        end = f.raw_bit_offset + f.raw_bit_length;
                                        cursor = end;
                                    }
                                    _ => break,
                                }
                            }
                            end
                        }
                        None => pi.raw_bit_offset + pi.raw_bit_length,
                    };
                    let extended_end_byte = (payload_start + (extended_end_bit + 7) / 8) as usize;
                    let extended_end = extended_end_byte.min(data.len());
                    if extended_end > start_byte {
                        let extended = data[start_byte..extended_end].to_vec();
                        log::info!("[extract] extended extraction for {}: {}B -> {}B (includes {} sockets)",
                            pi.item.code, parent_bits.len(), extended.len(), pi.item.socketed_items.len());
                        extended
                    } else {
                        parent_bits
                    }
                }
            } else if pi.item.flags.simple_item() {
                crate::protocol::d2i::parser::encode_item_to_jm_bits(pi).unwrap_or_default()
            } else {
                let table = crate::protocol::d2i::jm_reader::get_cached_stat_table();
                crate::protocol::d2i::parser::encode_item_with_sockets(pi, table).unwrap_or_default()
            },
            raw_bit_length: 0,
            // item_json 与存储工作台 warehouse_deposit 格式一致
            item_json: {
                let inv_size = crate::protocol::d2i::legacy::item::get_item_inventory_size(&pi.item.code);
                serde_json::json!({
                    "item_type": pi.item.code,
                    "quality": pi.item.quality.as_u8(),
                    "amount": pi.item.amount.max(1),
                    "simple_item": pi.item.flags.simple_item(),
                    "identified": pi.item.flags.identified(),
                    "socketed": pi.item.flags.socketed(),
                    "ethereal": pi.item.flags.ethereal(),
                    "position_x": pi.item.x,
                    "position_y": pi.item.y,
                    "inv_width": inv_size.0,
                    "inv_height": inv_size.1,
                    "stat_lists": pi.item.stat_lists,
                    "socketed_items": pi.item.socketed_items,
                }).to_string()
            },
            stash_name: Some(save_path.clone()),
            imported_at: chrono::Utc::now().to_rfc3339(),
            page_name: page_name.clone(),
            tags: String::new(),
            notes: String::new(),
            source_character: Some(source_char.clone()),
            source_save_path: Some(save_path.clone()),
            slot_equipped: slot.map(|s| s.to_string()),
            page_index: 0,
            position_x: 0,
            position_y: 0,
            inv_width: 1,
            inv_height: 1,
        };

        // 3. Store via WarehouseRepo
        let repos = db.repos();
        match repos.warehouse.add(&wi) {
            Ok(_) => {
                if is_equipped { equipped_count += 1; }
                else if pi.item.mode == ItemMode::Belt { belt_count += 1; }
                else { backpack_count += 1; }
                warehouse_ids.push(wi.id.clone());
            }
            Err(e) => {
                skipped.push(crate::commands::stash::SkippedItemReason {
                    item_name: code.clone(),
                    reason: format!("db error: {}", e),
                });
            }
        }
    }

    Ok(crate::commands::stash::ExtractResult {
        extracted_count: warehouse_ids.len(),
        warehouse_ids,
        page_name: page_name.clone(),
        source_character: source_char,
        equipped_count,
        backpack_count,
        belt_count,
        skipped_items: skipped,
    })
}

/// 调试工具: 输出 d2s 物品列表表格 (匹配 Python cli_construct --bits -1 输出)。
#[tauri::command]
pub fn debug_item_table(path: String) -> Result<String, String> {
    let data = std::fs::read(&path).map_err(|e| format!("读取失败: {}", e))?;

    // 使用 D2S 原生的 10-bit sorted code 解析器(不走 d2i parser)
    let items = crate::protocol::d2s::items::read_standard_items(&data)
        .map_err(|e| format!("解析失败: {}", e))?;

    // 总数量 = JM header u16 @ offset+2
    let m = crate::protocol::d2s::parser::marker_offsets(&data);
    let jm_offset = m.first_jm.unwrap_or(0);
    let total_count = if jm_offset + 4 <= data.len() {
        u16::from_le_bytes([data[jm_offset + 2], data[jm_offset + 3]]) as usize
    } else {
        items.len()
    };

    let mut lines = Vec::new();
    lines.push("# Flags 含义: I=已辨识  So=已镶嵌  N=新建  S=起始物品  C=压缩存储  E=无形  P=已打孔  W=符文之语  bit23=3(未知)".to_string());
    lines.push(format!("{} {:>4}  {:>4}  ver  {:>2} {:>2}  {:>6}  {:10}  {:28}  {:>10}  stats",
        " [idx]    ", "off", "len", "x", "y", "pos", "code", "name", "flags"));

    for (idx, pi) in items.iter().enumerate() {
        let it = &pi.item;
        let kind = if it.flags.is_ear() { "E" } else if it.flags.simple_item() { "C" } else { "F" };
        let sc: usize = it.stat_lists.iter().map(|sl| sl.stats.len()).sum();
        let raw_len = pi.raw_bit_length / 8;

        // Page name + belt position (matching Python PAGE_NAMES_ZH)
        let page_name = match it.mode {
            crate::protocol::common::ItemMode::Equipped => "身上",
            crate::protocol::common::ItemMode::Belt => {
                // Python checks page_name == "身上" && mode == 2
                // D2S belt items have mode Belt directly
                "腰带"
            },
            crate::protocol::common::ItemMode::Stored => {
                match it.page {
                    Some(crate::protocol::common::ItemPage::Backpack) => "背包",
                    Some(crate::protocol::common::ItemPage::Mod(4)) => "盒子",
                    Some(crate::protocol::common::ItemPage::Mod(5)) => "仓库",
                    _ => "背包",
                }
            },
            _ => "未知",
        };
        let belt_pos = if it.mode == crate::protocol::common::ItemMode::Belt {
            format!("[排{},位{}]", (it.x / 4) + 1, (it.x % 4) + 1)
        } else {
            String::new()
        };

        let fl = it.flags.raw;
        let mut flag_parts = Vec::new();
        if fl & (1<<4) != 0 { flag_parts.push("I"); }
        if fl & (1<<11) != 0 { flag_parts.push("So"); }
        if fl & (1<<13) != 0 { flag_parts.push("N"); }
        if fl & (1<<17) != 0 { flag_parts.push("S"); }
        if fl & (1<<21) != 0 { flag_parts.push("C"); }
        if fl & (1<<22) != 0 { flag_parts.push("E"); }
        if fl & (1<<23) != 0 { flag_parts.push("3"); }
        if fl & (1<<24) != 0 { flag_parts.push("P"); }
        if fl & (1<<26) != 0 { flag_parts.push("W"); }
        let flag_str = if flag_parts.is_empty() { "-".to_string() } else { flag_parts.join("+") };

        let code_disp = if it.code.is_empty() { "(空)".to_string() } else { it.code.clone() };

        let en_name = crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP
            .iter().find(|(c, _, _, _)| *c == it.code.as_str())
            .map(|(_, n, _, _)| n.to_string())
            .unwrap_or_default();
        let lv = if it.item_level > 0 { format!("({})", it.item_level) } else { String::new() };
        let disp = format!("{}{}{}", en_name, lv, belt_pos);

        lines.push(format!("  [{:2}] {} off={:3}B  len={:2}B  ver={}  x={:2} y={:2}  {:>6}  {:10}  {:28}  {:>8}  stats={}",
            idx, kind,
            pi.raw_bit_offset / 8,
            raw_len,
            it.version_raw & 7,
            it.x, it.y,
            page_name,
            code_disp,
            disp,
            flag_str,
            sc,
        ));
    }

    lines.push(format!("已找到/全部: {}/{}", items.len(), total_count));
    Ok(lines.join("\n"))
}

// ═══════════════════════════════════════════════════════════════════════════════
// list_characters_brief — 轻量级角色列表（只读 header，不解析物品）
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBriefInfo {
    pub name: String,
    pub class_en: String,
    pub class_cn: String,
    pub level: u8,
    pub is_hardcore: bool,
    pub is_dead: bool,
    pub is_expansion: bool,
    pub file_hash: String,
    /// Unix 时间戳，用于按更新时间排序
    pub save_timestamp: u32,
}

/// 列出所有角色及其简要信息（职业、等级、模式、文件哈希）。
/// 仅读取 d2s header，不解析物品/装备，适合列表缓存刷新。
#[tauri::command]
pub fn list_characters_brief(dir: String) -> Result<Vec<CharacterBriefInfo>, String> {
    let path = std::path::Path::new(&dir);
    if !path.is_dir() {
        return Ok(Vec::new()); // 目录不存在 → 空列表,而非 Err
    }
    let mut chars = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "d2s") {
            match read_single_character_brief(&p) {
                Ok(info) => chars.push(info),
                Err(e) => log::warn!("跳过 {}: {}", p.display(), e),
            }
        }
    }
    // 按保存时间降序排列（最新在前面）
    chars.sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));
    Ok(chars)
}

fn read_single_character_brief(p: &std::path::Path) -> Result<CharacterBriefInfo, String> {
    use crate::protocol::d2s::header::D2SHeader;

    let data = std::fs::read(p).map_err(|e| format!("读取失败: {}", e))?;
    if data.len() < 0x1C {
        return Err("文件过短".into());
    }

    // 复用 D2SHeader 解析器，不走完整 parse_d2s（太重）
    let header = D2SHeader::from_bytes(&data).map_err(|e| format!("header 解析失败: {:?}", e))?;
    // level 在 0x1B，D2SHeader 未包含此字段，直接字节读取
    let level = data[0x1B];

    // 角色名直接用文件名，与 list_characters 一致
    let name = p.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // 全文件 SHA256 哈希
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let file_hash = hex::encode(hasher.finalize());

    let class_en = header.character_class().name_en().to_string();
    let class_cn = header.character_class().name_cn().to_string();

    Ok(CharacterBriefInfo {
        name,
        class_en,
        class_cn,
        level,
        is_hardcore: header.is_hardcore(),
        is_dead: header.is_dead(),
        is_expansion: header.is_expansion(),
        file_hash,
        save_timestamp: header.save_timestamp,
    })
}
fn build_merc_equipment(
    merc: &[crate::protocol::d2i::parser::ParsedItem],
    resolver_opt: Option<&(Connection, Arc<NameResolver>)>,
    language: &str,
) -> Vec<EquipmentSlotInfo> {
    log::info!("[merc] build_merc_equipment: merc items count={}", merc.len());
    for (i, pi) in merc.iter().enumerate() {
        log::info!("[merc]   [{}] code={} location={:?} x={} y={}", i, pi.item.code, pi.item.location, pi.item.x, pi.item.y);
    }
    use crate::resource::TooltipFormatter;
    use crate::protocol::common::ItemLocation;

    // D2R mercenaries have 10 gear slots (original 4 + 6 added in D2R)
    let merc_slots: [(&str, ItemLocation); 10] = [
        ("helm", ItemLocation::Head),
        ("amulet", ItemLocation::Neck),
        ("ring_l", ItemLocation::LeftFinger),
        ("ring_r", ItemLocation::RightFinger),
        ("armor", ItemLocation::Torso),
        ("weapon_main", ItemLocation::RightHand),
        ("shield_main", ItemLocation::LeftHand),
        ("gloves", ItemLocation::Hands),
        ("boots", ItemLocation::Feet),
        ("belt", ItemLocation::Waist),
    ];

    let result: Vec<EquipmentSlotInfo> = merc_slots.iter().map(|(slot, loc)| {
            let item = merc.iter().find(|pi| pi.item.location == *loc);
            let code = item.map(|v| v.item.code.as_str());
            let quality_byte = item.map(|v| v.item.quality.as_u8());

            let (name_en, name_zh, name_zh_tw) = if let (Some(code), Some(item), Some((conn, resolver))) = (code, item, resolver_opt) {
                let en = resolver.resolve_with_affix(conn, code, quality_byte, item.item.unique_id, item.item.set_id, None, None, &[], "enUS");
                let zh = resolver.resolve_with_affix(conn, code, quality_byte, item.item.unique_id, item.item.set_id, None, None, &[], "zhCN");
                let tw = resolver.resolve_with_affix(conn, code, quality_byte, item.item.unique_id, item.item.set_id, None, None, &[], "zhTW");
                (Some(en.display_name), Some(zh.display_name), Some(tw.display_name))
            } else {
                let en = code.and_then(|c| crate::data::items::ITEM_CODE_MAP.iter().find(|e| e.0 == c).map(|e| e.1.to_string()));
                let fallback = code.map(|c| c.to_string());
                (en.clone(), fallback.clone(), fallback)
            };

            // 符文之语名称追加 [xxx] (与主角装备一致; 模糊匹配见 runewords::match_runeword_fuzzy)
            let (name_en, name_zh, name_zh_tw) = {
                let (mut ne, mut nz, mut ntw) = (name_en, name_zh, name_zh_tw);
                if let Some(item) = item
                    && item.item.flags.is_runeword()
                    && !item.item.socketed_items.is_empty() {
                    let rc: Vec<&str> = item.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
                    if let Some(rw_en) = crate::data::runewords::match_runeword_fuzzy(&rc, item.item.num_sockets) {
                        ne = Some(format!("{} [{}]", ne.as_deref().unwrap_or("?"), rw_en));
                        if let Some(rw_zh) = crate::data::runewords::match_runeword_fuzzy_zh(&rc, item.item.num_sockets) {
                            nz = Some(format!("{}[{}]", nz.as_deref().unwrap_or("?"), rw_zh));
                            ntw = Some(format!("{}[{}]", ntw.as_deref().unwrap_or("?"), rw_zh));
                        }
                    }
                }
                (ne, nz, ntw)
            };

            let main_stats: Vec<_> = item.map_or(Vec::new(), |v|
                v.item.stat_lists.first().map(|sl| sl.stats.clone()).unwrap_or_default()
            );
            let runeword_stats: Vec<_> = item.map_or(Vec::new(), |v|
                v.item.stat_lists.get(1).map(|sl| sl.stats.clone()).unwrap_or_default()
            );
            let _runeword_id = item.and_then(|v| if v.item.flags.is_runeword() { v.item.unique_id } else { None });
            let socketed = item.is_some_and(|v| v.item.flags.socketed());

            // Build tooltip for merc equipment
            let mut tooltip_data = item.map(|it| {
                let all_stats: Vec<_> = it.item.stat_lists.iter()
                    .flat_map(|sl| sl.stats.iter().cloned()).collect();
                let rw_stats: Vec<_> = if it.item.flags.is_runeword() && it.item.stat_lists.len() > 1 {
                    it.item.stat_lists[1..].iter()
                        .flat_map(|sl| sl.stats.iter().cloned()).collect()
                } else { Vec::new() };
                let lines = TooltipFormatter::equipment_tooltip(
                    slot, &it.item.code, it.item.quality.as_u8(),
                    Some(it.item.item_level),
                    name_en.as_deref(), name_zh.as_deref(),
                    None, it.item.unique_id, it.item.set_id,
                    &all_stats, &rw_stats,
                    it.item.max_durability, it.item.current_durability,
                    it.item.defense,
                    language, resolver_opt.map(|(c, _)| c), resolver_opt.map(|(_, r)| r.profile_id).unwrap_or(0),
                    false,
                );
                classify_tooltip(&lines)
            });

            // 底材防御已由 equipment_tooltip 附加 (含需求) — 2026-07-31 统一
            // 装备位 sockets 填装(装备在槽位上时也可能有孔)
            if let Some(it) = item
                && it.item.num_sockets > 0 {
                    let socketed_infos: Vec<SocketedItemInfo> = it.item.socketed_items.iter().map(|si| {
                        let q = si.quality.as_u8();
                        SocketedItemInfo {
                            code: si.code.clone(),
                            name_zh: None,
                            name_en: None,
                            quality: Some(q).filter(|&v| (2..=8).contains(&v)),
                            amount: si.amount,
                        }
                    }).collect();
                    if let Some(ct) = &mut tooltip_data {
                        ct.sockets = Some(SocketsInfo { count: it.item.num_sockets, items: socketed_infos });
                    }
                }

            EquipmentSlotInfo {
                slot: (*slot).to_string(),
                occupied: item.is_some(),
                code: code.map(String::from),
                name_zh, name_en, name_zh_tw,
                quality: quality_byte.and_then(quality_key_from_byte),
                socketed,
                skill_bonuses: {
                    let rp = resolver_opt.map(|(c, r)| (c, r.as_ref()));
                    let mut b = extract_skill_bonuses_with_opt(&main_stats, rp);
                    b.extend(extract_skill_bonuses_with_opt(&runeword_stats, rp));
                    b
                },
                durability_cur: item.map_or(0, |v| v.item.current_durability),
                durability_max: item.map_or(0, |v| v.item.max_durability),
                stats: item.map_or(ItemStats::default(), |v| {
                    categorize_item_stats(v.item.quality.as_u8(), v.item.flags.is_runeword(), &v.item.stat_lists)
                }),
                tooltip: tooltip_data,
            }
        }).collect();
    let occupied = result.iter().filter(|s| s.occupied).count();
    log::info!("[merc] build_merc_equipment done: {} slots total, {} occupied", result.len(), occupied);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::d2s::header::D2S_MAGIC;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_debug_item_table_xieedi() {
        let path = fixture_path("xieedi.d2s");
        if !path.exists() { return; }
        let output = debug_item_table(path.to_string_lossy().to_string()).unwrap();
        println!("\n=== D2S native parser output ===\n{}", output);
        assert!(output.contains("已找到/全部:"), "应包含总数行");
        assert!(output.contains("[idx]"), "应包含表头");
        assert!(output.contains("off="), "应包含物品行");
    }

    #[test]
    fn test_debug_item_table_kaixinxiedi() {
        let path = std::path::Path::new(r"D:\work_space\personal_workspace\d2r\开心邪帝.d2s");
        if !path.exists() { return; }
        let output = debug_item_table(path.to_string_lossy().to_string()).unwrap();
        println!("\n=== Rust output ===\n{}", output);
        assert!(output.starts_with("# Flags"));
        assert!(output.contains("已找到/全部:"));
        let n: usize = output.lines().filter(|l| l.contains("off=")).count();
        assert!(n >= 95, "Expected >=95 items, got {}", n);
    }
    fn make_minimal_d2s_bytes() -> Vec<u8> {
        let mut data = vec![0u8; 0x69];
        // 标准 D2R v105 header
        data[0..4].copy_from_slice(D2S_MAGIC);             // 0x00..0x04
        data[4..8].copy_from_slice(&0x69u32.to_le_bytes()); // 0x04..0x08 header_size
        // 0x08..0x0C filesize, 0x0C..0x10 checksum (0)
        data[0x10..0x14].copy_from_slice(&1u32.to_le_bytes()); // 0x10..0x14 active_weapon
        // 0x14..0x18 menu_layout (0)
        data[0x18] = 6;    // 0x18 class = Assassin
        data[0x19] = 0x10; // 0x19 status = expansion
        // 0x1A num_skills = 0
        data[0x1B] = 45;   // 0x1B level = 45
        // 0x1C..0x20 reserved, 0x20 save_timestamp, 0x24 unused, 0x28 hotkeys...
        // Fill with zeros up to ATTRIBUTES_OFFSET (0x341)
        while data.len() < crate::protocol::d2s::ATTRIBUTES_OFFSET {
            data.push(0);
        }
        // Attributes section: gf header + 9-bit header (level=12) + 7-bit value (45) + 9-bit trailer (0x1FF)
        data.extend_from_slice(&[0x67, 0x66]);
        // Bit encoding (LSB-first):
        //   header=12=0b000001100 (9 bits), value=45=0b0101101 (7 bits), trailer=0b111111111 (9 bits)
        //   Total 25 bits → 4 bytes
        let mut buf: u32 = 0;
        let mut pos = 0;
        for i in 0..9 {
            if (12u32 >> i) & 1 != 0 {
                buf |= 1 << pos;
            }
            pos += 1;
        }
        for i in 0..7 {
            if (45u32 >> i) & 1 != 0 {
                buf |= 1 << pos;
            }
            pos += 1;
        }
        for _ in 0..9 {
            buf |= 1 << pos;
            pos += 1;
        }
        for i in 0..4 {
            data.push(((buf >> (i * 8)) & 0xFF) as u8);
        }
        data
    }

    #[test]
    fn test_character_to_result_conversion() {
        let bytes = make_minimal_d2s_bytes();
        let f = parse_d2s(&bytes).unwrap();
        let result = character_to_result(
            &f,
            "/tmp/test.d2s",
            empty_equipment_slots(),
            build_binary_structure("standard-v105", f.header.active_weapon),
            "",
            None,
            "zhCN",
        );
        assert_eq!(result.level, 45);
        assert_eq!(result.class_en, "Assassin");
        assert_eq!(result.class_cn, "刺客");
        assert_eq!(result.class_zh_tw, "刺客");
        assert!(result.is_expansion);
        assert!(!result.is_hardcore);
        assert_eq!(result.source_path, "/tmp/test.d2s");
        assert_eq!(result.equipment.len(), 12);
        assert!(result.equipment.iter().all(|s| !s.occupied));
        assert!(!result.is_modified_layout);
    }

    #[test]
    fn test_equipment_slots_layout_12_slots() {
        let slots = empty_equipment_slots();
        let names: Vec<&str> = slots.iter().map(|s| s.slot.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "helm",
                "amulet",
                "ring_l",
                "ring_r",
                "armor",
                "weapon_main",
                "shield_main",
                "weapon_alt",
                "shield_alt",
                "gloves",
                "boots",
                "belt",
            ]
        );
    }

    #[test]
    fn test_read_character_info_happy_librarian() {
        let path = fixture_path("happy_librarian.d2s");
        if !path.exists() { eprintln!("SKIP: fixture happy_librarian.d2s 缺失"); return; }
        let result = read_character_info_inner(path.to_string_lossy().as_ref(), None, 0, "zhCN", None)
            .expect("read_character_info should succeed");

        assert_eq!(result.name, "开心图书馆长", "优先取 mod 扩展名");
        assert!(!result.is_modified_layout);
        assert_eq!(result.class_en, "Warlock");
        assert_eq!(result.class_cn, "术士");
        assert_eq!(result.level, 2);
        let occupied_count = result.equipment.iter().filter(|s| s.occupied).count();
        assert!(occupied_count <= 2, "Lv2 角色装备不宜过多 (实际 {} 件)", occupied_count);
    }

    #[test]

    fn test_read_character_info_old_xieedi_skipped() {
        let path = fixture_path("xieedi.d2s");
        if !path.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let result = read_character_info_inner(path.to_string_lossy().as_ref(), None, 0, "zhCN", None)
            .expect("read_character_info should succeed");
        assert_eq!(result.name, "开心邪帝", "优先取 mod 扩展名");
        assert_eq!(result.class_en, "Warlock");
    }
#[test]
    fn test_read_character_info_standard_tc03() {
        let path = fixture_path("standard_test_warlock_tc03.d2s");
        let result = read_character_info_inner(path.to_string_lossy().as_ref(), None, 0, "zhCN", None)
            .expect("read_character_info should succeed on standard layout");

        assert_eq!(result.name, "TestWarlock", "优先取 mod 扩展名");
        assert!(!result.is_modified_layout);
        assert_eq!(result.class_en, "Warlock");
        assert_eq!(result.class_cn, "术士");
        assert_eq!(result.class_zh_tw, "術士");
        assert_eq!(result.level, 12);
        assert_eq!(result.binary_structure.detected_layout, "standard-v105");
        // fixture 文件当前已装备状态
        let occupied: Vec<&EquipmentSlotInfo> =
            result.equipment.iter().filter(|s| s.occupied).collect();
        // d2s 装备解析走 items（d2s 专用解析器）
        assert_eq!(result.equipment.len(), 12);
        assert_eq!(occupied.len(), 7, "fixture 7 件已装备");

        let by_slot: std::collections::HashMap<&str, &EquipmentSlotInfo> =
            result.equipment.iter().map(|s| (s.slot.as_str(), s)).collect();
        assert_eq!(by_slot["helm"].code.as_deref(), Some("skp"));
        assert!(!by_slot["amulet"].occupied);
        assert!(!by_slot["ring_l"].occupied);
        assert_eq!(by_slot["ring_r"].code.as_deref(), Some("rin"));
        assert_eq!(by_slot["armor"].code.as_deref(), Some("qui"));
        assert_eq!(by_slot["weapon_main"].code.as_deref(), Some("spc"));
        assert!(!by_slot["shield_main"].occupied);
        assert!(!by_slot["weapon_alt"].occupied);
        assert!(!by_slot["shield_alt"].occupied);
        assert_eq!(by_slot["gloves"].code.as_deref(), Some("lgl"));
        assert_eq!(by_slot["boots"].code.as_deref(), Some("lbt"));
        assert_eq!(by_slot["belt"].code.as_deref(), Some("vbl"));
    }

    // ── Phase L (2026-07-09) SkillBonus 提取单元测试 ──

    use crate::protocol::common::ItemStat;

    fn stat(id: u16, skill_tab: Option<u8>, skill_level: Option<u16>, skill_id: Option<u16>, max_charges: Option<u8>, value: i64) -> ItemStat {
        ItemStat {
            id, value,
            param: skill_id.unwrap_or(0) as u32,
            skill_tab, skill_level, skill_id, max_charges,
        }
    }

    #[test]
    fn test_extract_skill_bonuses_skill_charges() {
        // 模拟 cm1 (地狱火炬) 23 charges of skill #64 (Blizzard), current=20
        let stats = vec![stat(204, None, Some(0), Some(64), Some(23), 20)];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 1);
        assert_eq!(bonuses[0].kind, "skill_charges");
        assert_eq!(bonuses[0].stat_id, 204);
        assert_eq!(bonuses[0].skill_id, Some(64));
        assert_eq!(bonuses[0].max_charges, Some(23));
        assert_eq!(bonuses[0].current_charges, Some(20));
        assert_eq!(bonuses[0].skill_level, Some(0));
        assert_eq!(bonuses[0].skill_tab, None);
        assert_eq!(bonuses[0].chance_pct, None);
    }

    #[test]
    fn test_extract_skill_bonuses_chance_to_cast() {
        // 模拟 +5% chance to cast Blizzard
        let stats = vec![stat(195, None, Some(0), Some(64), None, 5)];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 1);
        assert_eq!(bonuses[0].kind, "chance_to_cast");
        assert_eq!(bonuses[0].stat_id, 195);
        assert_eq!(bonuses[0].skill_id, Some(64));
        assert_eq!(bonuses[0].chance_pct, Some(5));
        assert_eq!(bonuses[0].max_charges, None);
        assert_eq!(bonuses[0].current_charges, None);
    }

    #[test]
    fn test_extract_skill_bonuses_skill_tab() {
        // 模拟 +1 暴风雪 (Sorceress Cold tab=0, +1 level)
        let stats = vec![stat(188, Some(0), Some(1), None, None, 1)];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 1);
        assert_eq!(bonuses[0].kind, "skill_tab");
        assert_eq!(bonuses[0].stat_id, 188);
        assert_eq!(bonuses[0].skill_tab, Some(0));
        assert_eq!(bonuses[0].skill_level, Some(1));
        assert_eq!(bonuses[0].skill_id, None);
    }

    #[test]
    fn test_extract_skill_bonuses_skips_normal_stats() {
        // 普通 stat (encode=0, descfunc=0) 不应产生任何 SkillBonus
        let stats = vec![
            stat(50, None, None, None, None, 5),    // lightmindam
            stat(54, None, None, None, None, 10),   // coldmindam
            stat(83, None, None, None, None, 1),    // magicfind
        ];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 0);
    }

    #[test]
    fn test_extract_skill_bonuses_mixed() {
        // 混合: 1 个 charges + 1 个 chance + 1 个 tab + 1 个普通
        let stats = vec![
            stat(204, None, Some(0), Some(64), Some(23), 20),       // skill_charges
            stat(195, None, Some(0), Some(64), None, 5),             // chance_to_cast
            stat(188, Some(0), Some(1), None, None, 1),              // skill_tab
            stat(50, None, None, None, None, 5),                     // 普通 stat
        ];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 3, "应跳过普通 stat, 只返回 3 个 skill bonus");
        assert_eq!(bonuses[0].kind, "skill_charges");
        assert_eq!(bonuses[1].kind, "chance_to_cast");
        assert_eq!(bonuses[2].kind, "skill_tab");
    }

    #[test]
    fn test_categorize_item_stats_normal_goes_to_base() {
        let stats = vec![
            ItemStat { id: 107, param: 395, value: 1, skill_id: Some(395), skill_level: Some(1), ..Default::default() },
        ];
        let sl = crate::protocol::common::StatList { stats };
        // normal quality → base
        // normal quality with skill_id → affix（mod 物品可能 normal 品质带技能）
        let result = categorize_item_stats(2, false, std::slice::from_ref(&sl));
        assert_eq!(result.affix.len(), 1, "skill stat → affix");
        assert_eq!(result.base.len(), 0, "normal 有 skill_id 不走 base");
        assert_eq!(result.affix[0].id, 107);
        assert_eq!(result.affix[0].skill_id, Some(395));
        // normal quality without skill_id → base
        let plain = vec![ItemStat { id: 16, param: 0, value: 39, ..Default::default() }];
        let sl_plain = crate::protocol::common::StatList { stats: plain };
        let result = categorize_item_stats(2, false, std::slice::from_ref(&sl_plain));
        assert_eq!(result.base.len(), 1, "无 skill_id → base");
        assert_eq!(result.affix.len(), 0);
        // magic quality → affix
        let result = categorize_item_stats(4, false, std::slice::from_ref(&sl));
        assert_eq!(result.base.len(), 0, "magic 不应有 base");
        assert_eq!(result.affix.len(), 1, "magic → affix");
        // set quality → affix + set_bonus
        let result = categorize_item_stats(5, false, std::slice::from_ref(&sl));
        assert_eq!(result.affix.len(), 1);
        assert_eq!(result.set_bonus.len(), 0, "只有一个 stat_list → 无 set_bonus");
        // runeword → affix
        let result = categorize_item_stats(2, true, std::slice::from_ref(&sl));
        assert_eq!(result.base.len(), 0, "runeword 不走 base");
        assert_eq!(result.affix.len(), 1, "runeword → affix");
    }

    #[test]
    fn test_extract_skill_bonuses_single_skill() {
        let stats = vec![
            ItemStat { id: 107, param: 395, value: 1, skill_id: Some(395), skill_level: Some(1), ..Default::default() },
        ];
        let bonuses = extract_skill_bonuses(&stats);
        assert_eq!(bonuses.len(), 1, "single_skill 应被提取");
        assert_eq!(bonuses[0].kind, "single_skill");
        assert_eq!(bonuses[0].stat_id, 107);
        assert_eq!(bonuses[0].skill_id, Some(395));
        assert_eq!(bonuses[0].skill_level, Some(1));
        assert_eq!(bonuses[0].chance_pct, None);
        assert_eq!(bonuses[0].max_charges, None);
    }

    /// 真实 fixture 端到端验证: EchoingStrike.d2s 通过 read_character_info
    /// 暴露 skill_bonuses (cm1 23-charges + xmg 命中时施法 + +skill_tab)
    #[test]
    fn test_echoing_strike_d2s_skill_bonuses_in_response() {
        let path = fixture_path("EchoingStrike.d2s");
        let result = read_character_info_inner(
            path.to_string_lossy().as_ref(),
            None,
            0,
            "zhCN",
            None,
        )
        .expect("read_character_info should succeed on standard layout");

        // 收集所有 skill_bonuses (从 equipment + backpack)
        let mut all_bonuses: Vec<&SkillBonus> = Vec::new();
        for slot in &result.equipment {
            all_bonuses.extend(slot.skill_bonuses.iter());
        }
        for item in &result.backpack_items {
            all_bonuses.extend(item.skill_bonuses.iter());
        }
        for item in &result.belt_items {
            all_bonuses.extend(item.skill_bonuses.iter());
        }
        eprintln!("EchoingStrike 提取到 {} 个 skill bonus:", all_bonuses.len());

        // EchoingStrike 是 Phase I 测试重点 (57 skill-related stats), 应该有不少 bonus
        assert!(
            all_bonuses.len() >= 3,
            "EchoingStrike 应提取 ≥3 个 skill bonus, 实际 {} 个",
            all_bonuses.len(),
        );

        // 至少应包含一种 kind (skill_charges/chance_to_cast/skill_tab)
        let kinds: std::collections::HashSet<&str> =
            all_bonuses.iter().map(|b| b.kind.as_str()).collect();
        assert!(
            !kinds.is_empty(),
            "应至少有一种 kind",
        );

        // 验证 SkillBonus 字段语义正确:
        // - skill_charges 同时有 skill_id + max_charges
        // - chance_to_cast 有 skill_id + chance_pct
        // - skill_tab 有 skill_tab + skill_level (但 skill_id 为 None)
        for b in &all_bonuses {
            match b.kind.as_str() {
                "skill_charges" => {
                    assert!(b.skill_id.is_some(), "skill_charges 缺 skill_id");
                    assert!(b.max_charges.is_some(), "skill_charges 缺 max_charges");
                    assert!(b.current_charges.is_some(), "skill_charges 缺 current_charges");
                }
                "chance_to_cast" => {
                    assert!(b.skill_id.is_some(), "chance_to_cast 缺 skill_id");
                    assert!(b.chance_pct.is_some(), "chance_to_cast 缺 chance_pct");
                }
                "skill_tab" => {
                    assert!(b.skill_tab.is_some(), "skill_tab 缺 skill_tab");
                    assert!(b.skill_level.is_some(), "skill_tab 缺 skill_level");
                    assert!(b.skill_id.is_none(), "skill_tab 不应有 skill_id");
                }
                other => panic!("未知 kind: {}", other),
            }
        }
    }
    /// 回归: standard_test_warlock_tc03.d2s 真实数据回归。
    ///
    /// 验证 parse_d2s → character_to_result 链路不会 panic，
    /// 且解析出的装备有合格数据（至少一件装备，stats 结构正确）。
    #[test]
    fn test_skill_bonuses_match_real_character_data() {
        let path = fixture_path("standard_test_warlock_tc03.d2s");
        let result = read_character_info_inner(
            path.to_string_lossy().as_ref(),
            None,
            0,
            "zhCN",
            None,
        )
        .expect("read_character_info should succeed on standard layout");

        eprintln!("TC03 equipment: {} slots", result.equipment.len());
        for slot in &result.equipment {
            if slot.occupied {
                eprintln!("  {}: code={:?} quality={:?} n_stats={}",
                    slot.slot, slot.code, slot.quality,
                    slot.stats.affix.len() + slot.stats.base.len()
                        + slot.stats.runeword.len() + slot.stats.set_bonus.len());
            }
        }

        // 不应有 fake/phantom skill_bonuses
        for slot in &result.equipment {
            for b in &slot.skill_bonuses {
                assert!(b.skill_id.is_some() || b.skill_tab.is_some() || b.max_charges.is_some(),
                    "每个 skill_bonus 必须有 skill_id/skill_tab/max_charges: {:?}", b);
            }
        }

        // 至少有 1 件装备
        let occupied = result.equipment.iter().filter(|s| s.occupied).count();
        assert!(occupied >= 1, "TC03 应有至少 1 件装备");
    }
}
