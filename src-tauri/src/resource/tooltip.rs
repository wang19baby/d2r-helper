//! Unified TooltipFormatter for D2R items.
//! 统一的 tooltip 格式化器，替代 commands/character.rs 中分散的
//! `format_equipment_stat_line`、`known_equipment_stats`、`build_equipment_tooltip_lines`，
//! 以及 stash.rs 中的内联 tooltip 构建。
//! ## 用法
//! ```ignore
//! // 装备 tooltip
//! let lines = TooltipFormatter::equipment_tooltip(
//!     "helm", "cap", 2, Some(85),
//!     Some("Cap"), Some("帽子"),
//!     None, None, None,
//!     &main_stats, &runeword_stats,
//!     0, 0,
//!     "zhCN",
//!     None, 0,
//! );
//! // 仓库 tooltip
//! let lines = TooltipFormatter::stash_tooltip(
//!     "艾尔", "El Rune", "r01", "rune",
//!     Some(2), 1, "zhCN", &[],
//! );
//! ```

use rusqlite::{Connection, params};
use crate::protocol::common::ItemStat;
use crate::resource::queries;

/// 统一的 tooltip 格式化器。
/// 所有方法均为静态（无状态），方便各处调用。
pub struct TooltipFormatter;

impl TooltipFormatter {
    // ── Public API ──

    /// 构建装备 tooltip（角色页）
    ///
    /// `conn` 和 `profile_id` 用于 `stat_def` 表回退查询（当静态 match 无法识别 stat 时）。
    pub fn equipment_tooltip(
        slot: &str,
        code: &str,
        quality_byte: u8,
        item_level: Option<u8>,
        name_en: Option<&str>,
        name_zh: Option<&str>,
        runeword_id: Option<u16>,
        unique_id: Option<u16>,
        set_id: Option<u16>,
        main_stats: &[ItemStat],
        runeword_stats: &[ItemStat],
        max_durability: u8,
        current_durability: u8,
        defense: u16,
        language: &str,
        conn: Option<&Connection>,
        profile_id: i64,
        show_slot: bool,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let title = name_zh.or(name_en).unwrap_or(code);
        let english = name_en.unwrap_or(code);

        // 1. 槽位（非空且 show_slot 时才显示）
        // 佣兵 tooltip 不显示槽位: 装备格子已由 UI 固定呈现
        if show_slot && !slot.is_empty() {
            lines.push(format!("{}: {}", Self::slot_label(language), Self::slot_name(slot, language)));
        }
        // 2. 中文名
        lines.push(title.to_string());
        // 3. 英文名（如果和中不同）
        if english != title {
            lines.push(english.to_string());
        }
        // 4. 代码
        lines.push(Self::meta_line("Code", code, language));
        // 5. 类型
        let kind = crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP
            .iter()
            .find(|(c, _, _, _)| *c == code)
            .map(|(_, _, k, _)| *k)
            .unwrap_or("misc");
        lines.push(Self::meta_line("Type", &type_name(kind, language, conn, profile_id), language));
        // 6. 品质
        if let Some(quality) = quality_name(quality_byte, language) {
            lines.push(Self::meta_line("Quality", quality, language));
        }
        // 7. 物品等级
        if let Some(ilvl) = item_level {
            lines.push(Self::meta_line("ItemLevel", &format!("{}", ilvl), language));
        }
        // 8. 耐久度
        if max_durability > 0 {
            match language {
                "enUS" => lines.push(format!("Durability: {}/{}", current_durability, max_durability)),
                _ => lines.push(format!("耐久度: {}/{}", current_durability, max_durability)),
            }
        }
        // 8b. 底材属性: 武器攻击力 / 护甲防御 / 需求 (与 stash/character 路径一致)
        // 武器攻击力 (双手武器取 2h 区间)
        if let Some(wpn) = crate::data::items_base::weapon_stats(code) {
            let (dmin, dmax) = wpn.display_damage();
            if dmin > 0 || dmax > 0 {
                let line = if dmin == dmax {
                    match language {
                        "enUS" => format!("Damage: {}", dmin),
                        _ => format!("攻击力: {}", dmin),
                    }
                } else {
                    match language {
                        "enUS" => format!("Damage: {}-{}", dmin, dmax),
                        _ => format!("攻击力: {}-{}", dmin, dmax),
                    }
                };
                lines.push(line);
            }
            if wpn.reqstr > 0 {
                let line = match language {
                    "enUS" => format!("Required Strength: {}", wpn.reqstr),
                    _ => format!("需要力量: {}", wpn.reqstr),
                };
                lines.push(line);
            }
            if wpn.reqdex > 0 {
                let line = match language {
                    "enUS" => format!("Required Dexterity: {}", wpn.reqdex),
                    _ => format!("需要敏捷: {}", wpn.reqdex),
                };
                lines.push(line);
            }
            if wpn.levelreq > 0 {
                let line = match language {
                    "enUS" => format!("Required Level: {}", wpn.levelreq),
                    _ => format!("需要等级: {}", wpn.levelreq),
                };
                lines.push(line);
            }
        }
        // 护甲防御 + 需求
        if let Some(def) = crate::data::items_base::armor_stats(code) {
            // 实际防御优先: 解析出的 Item.defense; 0 时回退底材区间
            if defense > 0 {
                let line = match language {
                    "enUS" => format!("Defense: {}", defense),
                    _ => format!("防御: {}", defense),
                };
                lines.push(line);
            } else if def.minac > 0 || def.maxac > 0 {
                let line = if def.minac == def.maxac {
                    match language {
                        "enUS" => format!("Defense: {}", def.minac),
                        _ => format!("防御: {}", def.minac),
                    }
                } else {
                    match language {
                        "enUS" => format!("Defense: {}-{}", def.minac, def.maxac),
                        _ => format!("防御: {}-{}", def.minac, def.maxac),
                    }
                };
                lines.push(line);
            }
            if def.reqstr > 0 {
                let line = match language {
                    "enUS" => format!("Required Strength: {}", def.reqstr),
                    _ => format!("需要力量: {}", def.reqstr),
                };
                lines.push(line);
            }
            if def.reqdex > 0 {
                let line = match language {
                    "enUS" => format!("Required Dexterity: {}", def.reqdex),
                    _ => format!("需要敏捷: {}", def.reqdex),
                };
                lines.push(line);
            }
            if def.levelreq > 0 {
                let line = match language {
                    "enUS" => format!("Required Level: {}", def.levelreq),
                    _ => format!("需要等级: {}", def.levelreq),
                };
                lines.push(line);
            }
        }
        // 盾牌 smite 伤害 (含圣骑士盾)
        if let Some((smin, smax)) = crate::data::items_base::shield_smite(code) {
            let line = if smin == smax {
                match language {
                    "enUS" => format!("Smite Damage: {}", smin),
                    _ => format!("盾击伤害: {}", smin),
                }
            } else {
                match language {
                    "enUS" => format!("Smite Damage: {}-{}", smin, smax),
                    _ => format!("盾击伤害: {}-{}", smin, smax),
                }
            };
            lines.push(line);
        }
        // 暗金物品等级需求 (unique_item_def.level_req)
        if quality_byte == 7
            && let Some(uid) = unique_id
            && let Some(conn) = conn
            && let Some(def) = crate::resource::queries::get_unique_def(conn, profile_id, uid)
            && def.level_req > 0 {
                let line = match language {
                    "enUS" => format!("Required Level: {}", def.level_req),
                    _ => format!("需要等级: {}", def.level_req),
                };
                lines.retain(|l| !l.starts_with("需要等级:") && !l.starts_with("Required Level:"));
                lines.push(line);
            }
        // 套装物品等级需求 (存档 set_id = setitems.txt *ID, 按 item_id 查)
        if quality_byte == 5
            && let Some(sid) = set_id
            && let Some(conn) = conn
            && let Some(def) = crate::resource::queries::get_set_item_by_item_id(conn, profile_id, sid)
            && def.level_req > 0 {
                let line = match language {
                    "enUS" => format!("Required Level: {}", def.level_req),
                    _ => format!("需要等级: {}", def.level_req),
                };
                lines.retain(|l| !l.starts_with("需要等级:") && !l.starts_with("Required Level:"));
                lines.push(line);
            }
        // 9. 词缀属性行
        for line in Self::format_stats(main_stats, runeword_stats, language, conn, profile_id) {
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        // 10. 已知特殊 stat
        for stat in Self::known_stats(code, unique_id, set_id, runeword_id) {
            if !lines.contains(&stat.to_string()) {
                lines.push(stat.to_string());
            }
        }
        // 11. 调试 ID
        if let Some(id) = unique_id {
            lines.push(Self::meta_line("Unique ID", &format!("{}", id), language));
        }
        if let Some(id) = set_id {
            lines.push(Self::meta_line("Set ID", &format!("{}", id), language));
        }
        if let Some(id) = runeword_id {
            lines.push(Self::meta_line("Runeword ID", &format!("{}", id), language));
        }
        lines
    }

    /// 构建仓库 tooltip（简单位/堆叠物品 + 装备物品的词缀）。
    /// 对有 stat_list 的非简单位，会追加格式化后的词缀行。
    pub fn stash_tooltip(
        name: &str,
        en_name: &str,
        code: &str,
        kind: &str,
        quality: Option<u8>,
        quantity: u32,
        language: &str,
        stats: &[ItemStat],
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let title = if language == "enUS" { name } else { name };
        lines.push(title.to_string());
        if en_name != name && language != "enUS" {
            lines.push(en_name.to_string());
        }
        if quantity > 1 {
            lines.push(format!("{}: {}", Self::qty_label(language), quantity));
        }
        lines.push(Self::meta_line("Type", kind, language));
        lines.push(Self::meta_line("Code", code, language));
        if let Some(q) = quality {
            let qn = quality_name(q, language).unwrap_or("normal");
            lines.push(Self::meta_line("Quality", qn, language));
        }

        // 非简单位 → 追加格式化后的词缀
        if !stats.is_empty() {
            let stat_lines = Self::format_stats(stats, &[], language, None, 0);
            if !stat_lines.is_empty() {
                lines.push(String::new()); // 分隔行
                lines.extend(stat_lines);
            }
        }

        lines
    }

    /// 格式化单个 stat 行
    ///
    /// 如果静态 match 无法识别该 stat ID，且提供了 `conn`，
    /// 则回退到查询 `stat_def` 表获取其名称。
    /// Query localized_string for a specific key, namespace, language.
    /// Returns None if not found.
    fn _get_localized_flex(conn: &Connection, profile_id: i64, key: &str, namespace: &str, language: &str) -> Option<String> {
        let mut stmt = conn.prepare_cached(
            "SELECT text_value FROM localized_string
             WHERE profile_id = ?1 AND namespace = ?2 AND string_key = ?3 AND language = ?4
             LIMIT 1",
        ).ok()?;
        stmt.query_row(params![profile_id, namespace, key, language], |row| row.get(0)).ok()
    }

    /// 格式化套装加成行: 每件数分组, 显示 "(2 件): 属性" 或 "(全套): 属性"。
    pub fn format_set_bonuses(
        bonuses: &[crate::resource::queries::SetBonusDefRow],
        language: &str,
        conn: Option<&Connection>,
        profile_id: i64,
    ) -> Vec<(u8, String)> {
        let mut out: Vec<(u8, String)> = Vec::new();
        let mut by_piece: std::collections::BTreeMap<u8, Vec<&crate::resource::queries::SetBonusDefRow>> =
            std::collections::BTreeMap::new();
        for b in bonuses {
            by_piece.entry(b.piece_count).or_default().push(b);
        }
        for (pieces, rows) in by_piece {
            let mut lines: Vec<String> = Vec::new();
            for b in rows {
                let stat = ItemStat {
                    id: b.stat_id,
                    param: b.param as u32,
                    value: b.min_value as i64,
                    skill_id: None,
                    skill_tab: None,
                    skill_level: None,
                    max_charges: None,
                };
                if let Some(line) = Self::format_stat(&stat, language, conn, profile_id) {
                    lines.push(line);
                }
            }
            if !lines.is_empty() {
                let prefix = match language {
                    "enUS" if pieces == 6 => "Complete Set:".to_string(),
                    "enUS" => format!("{} pieces:", pieces),
                    _ if pieces == 6 => "全套:".to_string(),
                    _ => format!("{} 件:", pieces),
                };
                out.push((pieces, format!("{} {}", prefix, lines.join(" / "))));
            }
        }
        out
    }

    /// 格式化单个 stat 为 tooltip 行。
    ///
    /// `collapsible_if` 例外: 内部 DB 优先/静态兜底/特判三分支共 65 行,
    /// 折叠成 let-chain 会产生大 diff, 维持嵌套可读性。
    #[allow(clippy::collapsible_if)]
    pub fn format_stat(
        stat: &ItemStat,
        language: &str,
        conn: Option<&Connection>,
        profile_id: i64,
    ) -> Option<String> {
        if stat.id == 107
            && let Some(c) = conn {
                let mut skill_en = queries::get_skill_def(c, profile_id, stat.param as u16)
                    .map(|s| s.name_en);
                if skill_en.is_none()
                    && let Ok(vp_id) = c.query_row(
                        "SELECT id FROM resource_profile WHERE profile_key = 'vanilla:d2r-92777' LIMIT 1",
                        [], |row| row.get::<_, i64>(0),
                    ) {
                        skill_en = queries::get_skill_def(c, vp_id, stat.param as u16)
                            .map(|s| s.name_en);
                    }
                if let Some(ref en) = skill_en {
                    let skill_key = format!("{}name", en.replace(' ', "").to_lowercase());
                    let localized = Self::_get_localized_flex(c, profile_id, &skill_key, "skills", language)
                        .or_else(|| {
                            if let Ok(vp_id) = c.query_row(
                                "SELECT id FROM resource_profile WHERE profile_key = 'vanilla:d2r-92777' LIMIT 1",
                                [], |row| row.get::<_, i64>(0),
                            ) {
                                Self::_get_localized_flex(c, vp_id, &skill_key, "skills", language)
                            } else { None }
                        });
                    let display = localized.unwrap_or_else(|| en.clone());
                    return Some(match language {
                        "enUS" => format!("+{} to {}", stat.value, display),
                        _ => format!("+{} {}", stat.value, display),
                    });
                } else {
                    return Some(match language {
                        "enUS" => format!("+{} to Skill {}", stat.value, stat.param),
                        _ => format!("+{} 技能{}", stat.value, stat.param),
                    });
                }
            }
        // DB 优先: stat_def 是 mod 真实定义 (mod 会重定义原版 stat id,
        // 静态 id 映射按原版写, mod 下会错位)。name_en(code) → 中文映射,
        // 无映射时显示原始 code。静态 id 映射仅作无 DB 时的兜底。
        if let Some(conn) = conn {
            if let Some(def) = queries::get_stat_def(conn, profile_id, stat.id)
                && !def.name_en.is_empty() {
                    // 特判: 充能技能 (item_charged_skill) — 查技能名
                    if def.name_en == "item_charged_skill" {
                        if let Some(skill) = queries::get_skill_def(conn, profile_id, stat.param as u16) {
                            return Some(match language {
                                "enUS" => format!("Charges: {} ({})", skill.name_en, stat.value),
                                _ => format!("充能: {} ({})", skill.name_en, stat.value),
                            });
                        }
                        return Some(match language {
                            "enUS" => format!("+{} Skill Charges", stat.value),
                            _ => format!("+{} 技能充能", stat.value),
                        });
                    }
                    // 特判: 技能树加成 (item_addskill_tab, stat 188)
                    // D2R 存档 param 编码 = (class_id << 3) | tab:
                    //   高 3 位 = CharacterClass enum (0=Amazon,1=Sorc,2=Necro,3=Pal,
                    //             4=Barb,5=Druid,6=Assassin,7=Warlock)
                    //   低 3 位 = tab index within class (0-2)
                    // 实测恶魔角锋: param=32/33/34 → class=4(Barb) tab=0/1/2 ✓
                    // tab 名查 localized_string skillcategory{abbr}{3-tab} (SkillPage 映射)
                    if def.name_en == "item_addskill_tab" {
                        let class_id = ((stat.param >> 3) & 0x7) as u8;
                        let tab = (stat.param & 0x7) as u8;
                        let abbr = match class_id {
                            0 => "am", 1 => "so", 2 => "ne", 3 => "pa",
                            4 => "ba", 5 => "dr", 6 => "as", 7 => "wa",
                            _ => "",
                        };
                        let tab_name = if !abbr.is_empty() && tab <= 2 {
                            let key = format!("skillcategory{}{}", abbr, 3 - tab);
                            Self::_get_localized_flex(conn, profile_id, &key, "skills", language)
                                .unwrap_or_else(|| format!("Skill Tab {}", tab))
                        } else {
                            format!("Skill Tab {}", tab)
                        };
                        return Some(match language {
                            "enUS" => format!("+{} to {}", stat.value, tab_name),
                            _ => format!("{} +{}", tab_name, stat.value),
                        });
                    }
                    // 特判: 灵气 (item_aura, stat 151) — param 是技能 id, 查 skillname{id}
                    // 例: 眼光 (Insight) 的冥想灵气 → "等级 16 冥想灵气"
                    if def.name_en == "item_aura" {
                        let key = format!("skillname{}", stat.param);
                        let aura_name = Self::_get_localized_flex(conn, profile_id, &key, "skills", language)
                            .or_else(|| queries::get_skill_def(conn, profile_id, stat.param as u16).map(|s| s.name_en))
                            .or_else(|| skill_name(stat.param).map(|s| s.to_string()))
                            .unwrap_or_else(|| "Aura".to_string());
                        return Some(match language {
                            "enUS" => format!("Level {} {} Aura", stat.value, aura_name),
                            _ => format!("等级 {} {}灵气", stat.value, aura_name),
                        });
                    }
                    let label = if language.starts_with("zh") {
                        Self::mod_stat_zh_label(&def.name_en)
                            .map(|s| format!("{} {}", s, stat.value))
                            .unwrap_or_else(|| format!("{} {}", def.name_en, stat.value))
                    } else {
                        format!("{} {}", def.name_en, stat.value)
                    };
                    return Some(label);
                }
        }
        // 静态 id 映射兜底 (无 DB / profile 缺失时, 按原版语义)
        let static_result = match language {
            "enUS" => Self::format_stat_en(stat),
            _ => Self::format_stat_zh(stat),
        };
        if static_result.is_some() {
            return static_result;
        }
        None
    }

    /// stat code (itemstatcost name_en) → 中文标签映射。
    /// DB stat_def 优先时使用: 与静态 id 映射不同, 这里按 code 匹配,
    /// mod 重定义 stat id 时依然正确。
    fn mod_stat_zh_label(name_en: &str) -> Option<&'static str> {
        // 标准 itemstatcost code (原版 D2R)
        Some(match name_en {
            // 基础属性/恢复
            "manarecoverybonus" => "法力恢复提高%",  // 27
            "manarecovery" => "法力恢复",            // 26
            "stamina" => "耐力",
            "maxstamina" => "最大耐力",
            "hpregen" => "回复生命",                // 74
            "staminarecoverybonus" => "耐力恢复速度",
            // 伤害
            "item_maxdamage_percent" => "最大伤害%", // 17
            "item_mindamage_percent" => "最小伤害%", // 18
            "damagepercent" => "增强伤害%",         // 25
            "item_armor_percent" => "防御强化%",    // 16
            "mindamage" => "最小伤害",              // 21
            "maxdamage" => "最大伤害",              // 22
            "item_mindamage" => "最小伤害",
            "item_maxdamage" => "最大伤害",
            "item_tohit" => "命中",                 // 24
            "tohit" => "命中",                      // 19
            "item_tohit_percent" => "命中%",
            // 生命/法力/防御
            "item_maxhp" => "生命上限",
            "item_maxmana" => "法力上限",
            "item_maxhp_percent" => "最大生命%",    // 76
            "item_maxmana_percent" => "最大法力%",  // 77
            "item_armor" => "防御",                 // 31
            "armorclass" => "防御",
            "item_lightradius" => "光照",           // 89
            // 速度/恢复
            "item_fasterattackrate" => "攻击速度%", // 93
            "item_fastergethitrate" => "快速打击恢复%", // 99
            "item_fastermovevelocity" => "高速跑步/行走%", // 96
            "item_fastercastrate" => "快速施法%",   // 105
            "item_fasterblockrate" => "快速格挡恢复%", // 102
            "item_levelreqpct" => "需求等级%",      // 94
            "item_req_percent" => "需求减少%",      // 164
            // 技能
            "item_allskills" => "所有技能",         // 116
            "item_single_skill" => "单技能加成",    // 107
            "item_nonclassskill" => "全职业技能",
            // 抗性
            "resist-fire" => "火抗%", "resist-cold" => "冰抗%", "resist-ltng" => "电抗%", "resist-pois" => "毒抗%",
            "maxfireresist" => "最大火抗", "maxcoldresist" => "最大冰抗", "maxltngresist" => "最大电抗", "maxpoisonresist" => "最大毒抗",
            // 其他常见
            "item_goldbonus" => "额外金币%",        // 79
            "item_magiconoff" => "魔法属性",
            "item_normaldamage_reduction" => "物理伤害减轻", // 36
            "item_magicdamage_reduction" => "魔法伤害降低", // 35
            "item_damagetargetac" => "降低目标防御",
            "item_knockback" => "击退",
            "item_freeze" => "冻结目标",
            "item_crushingblow" => "压碎打击%",
            "item_openwounds" => "撕开伤口",
            "item_preventheal" => "阻止怪物治疗",   // 117
            "item_restinpeace" => "怪物安息",       // 108
            "item_ignoretargetac" => "无视目标防御",
            "item_indesctructible" => "无法破坏",
            // ── 补全: 词缀文件/stat_def 全部出现的物品属性 (2026-07-31) ──
            // 基础属性/资源
            "strength" => "力量", "dexterity" => "敏捷", "energy" => "能量", "vitality" => "体力",
            "maxhp" => "最大生命", "maxmana" => "最大法力", "mana" => "法力", "gold" => "金币",
            "durability" => "耐久度", "maxdurability" => "最大耐久度",
            "newskills" => "新技能点数", "experience" => "经验",
            // 抗性
            "fireresist" => "火抗%", "coldresist" => "冰抗%", "lightresist" => "电抗%", "poisonresist" => "毒抗%",
            "maxlightresist" => "最大电抗", "magic_damage_reduction" => "魔法伤害减轻", "normal_damage_reduction" => "物理伤害减轻",
            // 伤害 (简写体系)
            "bonus_maxdamage" => "附加最大伤害", "bonus_mindamage" => "附加最小伤害",
            "magicmaxdam" => "最大魔法伤害", "magicmindam" => "最小魔法伤害",
            "burningmax" => "最大燃烧伤害", "burningmin" => "最小燃烧伤害",
            "coldlength" => "冰冻持续时间", "poisonlength" => "毒素持续时间", "poison_count" => "毒素层数",
            "hp-kill" => "杀敌回血", "mana-lost" => "法力损耗",
            // 命中/防御修正
            "item_undead_tohit" => "对不死生物命中", "item_demon_tohit" => "对恶魔命中",
            "item_tohitpercent_perlevel" => "每级准确率%", "item_ac_percent_vs_monster" => "对怪物防御%",
            "item_damage_percent_vs_monster" => "对怪物伤害%", "toblock" => "格挡率", "velocitypercent" => "移动速度%",
            // 伤害随时间/每级
            "item_maxdamage_bytime" => "最大伤害(持续)", "item_maxdamage_percent_bytime" => "增强伤害%(持续)",
            "item_maxdamage_percent_perlevel" => "每级增强伤害%", "item_maxdamage_perlevel" => "每级最大伤害",
            "item_hp_perlevel" => "每级生命", "item_mana_perlevel" => "每级法力",
            "item_strength_perlevel" => "每级力量", "item_dexterity_perlevel" => "每级敏捷",
            "item_energy_perlevel" => "每级能量", "item_vitality_perlevel" => "每级体力",
            "item_strength_bytime" => "力量(持续)", "item_vitality_bytime" => "体力(持续)",
            "item_tohit_bytime" => "命中(持续)", "item_tohit_undead_bytime" => "对不死命中(持续)",
            // 技能类
            "item_singleskill" => "单技能加成", "item_addskill_tab" => "技能树加成", "item_addclassskills" => "职业全技能",
            "item_aura" => "灵气", "item_skillonattack" => "攻击时释放技能", "item_skillonhit" => "击中时释放技能",
            "item_skillonkill" => "杀死时释放技能", "item_skillondeath" => "死亡时释放技能", "item_skillongethit" => "被击中时释放技能",
            "item_extra_charges" => "额外充能次数", "item_charge_noconsume" => "充能不消耗", "item_noconsume" => "不消耗",
            // 特殊效果
            "item_cannotbefrozen" => "无法冰冻", "item_halffreezeduration" => "冰冻时间减半",
            "item_deadlystrike" => "致命一击%", "item_deadlystrike_bytime" => "致命一击%(持续)", "item_deadlystrike_perlevel" => "每级致命一击%",
            "item_crushingblow_perlevel" => "每级压碎打击%", "item_openwounds_bytime" => "撕开伤口(持续)", "item_openwounds_perlevel" => "每级撕开伤口%",
            "item_damagetomana" => "伤害转化为法力%", "item_slow" => "减慢目标%", "item_stupidity" => "击中使目标致盲",
            "item_howl" => "击退并吓跑怪物", "item_thorns_perlevel" => "每级荆棘伤害", "item_staminadrainpct" => "耐力消耗%", "item_magicbonus" => "魔法属性",
            // 掉落/经济
            "item_find_gems_perlevel" => "每级宝石掉落%", "item_find_gold_bytime" => "金币(持续)", "item_find_gold_perlevel" => "每级金币%",
            "item_find_item" => "更佳机会取得物品%", "item_addexperience" => "额外经验%", "item_reducedprices" => "商店价格降低%",
            "item_extrablood" => "额外血液", "item_manaafterkill" => "杀敌回蓝", "item_doubleherbduration" => "药水持续时间%",
            // 孔/耐久恢复
            "item_numsockets" => "孔数", "item_replenish_durability" => "自动恢复耐久", "item_replenish_quantity" => "自动恢复数量",
            "item_levelreq" => "需求等级", "item_throwable" => "可投掷", "item_kick_damage_bytime" => "踢击伤害(持续)", "item_kick_damage_perlevel" => "每级踢击伤害",
            // 吸收 (旧命名, 无下划线)
            "item_absorbfire" => "火焰吸收", "item_absorbcold" => "冰冷吸收", "item_absorblight" => "闪电吸收", "item_absorbmagic" => "魔法吸收",
            "item_absorbfire_percent" => "火焰吸收%", "item_absorbcold_percent" => "冰冷吸收%", "item_absorblight_percent" => "闪电吸收%", "item_absorbmagic_percent" => "魔法吸收%",
            "item_absorb_crush_percent" => "碾压吸收%", "item_absorb_slash_percent" => "劈砍吸收%", "item_absorb_thrust_percent" => "穿刺吸收%",
            "item_absorb_pois_perlevel" => "每级毒素吸收", "item_absorb_pois_bytime" => "毒素吸收(持续)",
            // mod 被动/免疫穿透
            "passive_avoid" => "闪避", "passive_evade" => "闪避(远程)", "passive_critical_strike" => "暴击", "passive_warmth" => "温暖(法力恢复)",
            "crit" => "暴击",
            "passive_mastery_melee_th" => "近战命中专精", "passive_mastery_noconsume" => "不消耗专精", "passive_dmg_pierce" => "伤害穿透",
            "passive_mastery_gethit_rate" => "受击专精",
            "item_pierce" => "穿透", "item_pierce_ltng" => "闪电穿透%", "item_pierce_fire_immunity" => "火焰免疫穿透", "item_pierce_cold_immunity" => "冰冷免疫穿透",
            "item_pierce_light_immunity" => "闪电免疫穿透", "item_pierce_poison_immunity" => "毒素免疫穿透", "item_pierce_magic_immunity" => "魔法免疫穿透", "item_pierce_damage_immunity" => "物理免疫穿透",
            "armor_override_percent" => "防御覆盖%", "item_armorpercent_bytime" => "防御%(持续)", "item_resist_fire_perlevel" => "每级火抗", "item_resist_ltng_perlevel" => "每级电抗",
            "item_resist_pois_perlevel" => "每级毒抗", "item_resist_pois_bytime" => "毒抗(持续)", "item_damage_demon_bytime" => "对恶魔伤害(持续)", "item_damage_demon_perlevel" => "每级对恶魔伤害",
            // 下面继续 mod 专属 (原有映射保留)
            // 元素穿透
        // d2emu 模组 stat 体系 (原映射)
            // 元素穿透
            "passive_fire_pierce" => "火焰穿透",
            "passive_ltng_pierce" => "闪电穿透",
            "passive_cold_pierce" => "冰冷穿透",
            "passive_pois_pierce" => "毒素穿透",
            "passive_mag_pierce" => "魔法穿透",
            "passive_phys_pierce" => "物理穿透",
            "item_pierce_fire" => "火焰穿透%",
            "item_pierce_light" => "闪电穿透%",
            "item_pierce_cold" => "冰冷穿透%",
            "item_pierce_pois" => "毒素穿透%",
            "item_pierce_magic" => "魔法穿透%",
            // 元素精通
            "passive_fire_mastery" => "火焰精通",
            "passive_ltng_mastery" => "闪电精通",
            "passive_cold_mastery" => "冰冷精通",
            "passive_pois_mastery" => "毒素精通",
            "passive_mag_mastery" => "魔法精通",
            "passive_summon_resist" => "召唤抗性",
            // 专精
            "passive_mastery_throw_dmg" => "投掷伤害",
            "passive_mastery_throw_crit" => "投掷暴击",
            "passive_mastery_throw_th" => "投掷准确率",
            "passive_mastery_melee_dmg" => "近战伤害",
            "passive_mastery_melee_crit" => "近战暴击",
            "passive_mastery_attack_speed" => "攻速专精",
            "passive_mastery_replenish_oncrit" => "暴击回能",
            "passive_mastery_item_req_percent" => "需求降低专精",
            "passive_mastery_item_level_req_percent" => "等级需求降低",
            "passive_weaponblock" => "武器格挡",
            "passive_dodge" => "闪避",
            // 元素伤害
            "item_fire_damagemax_bytime" => "火焰持续伤害",
            "item_ltng_damagemax_bytime" => "闪电持续伤害",
            "item_cold_damagemax_bytime" => "冰冷持续伤害",
            "item_pois_damagemax_bytime" => "毒素持续伤害",
            "item_magic_damagemax_bytime" => "魔法持续伤害",
            "item_cold_damagemax_perlevel" => "每级冰冷伤害上限",
            "item_ltng_damagemax_perlevel" => "每级闪电伤害上限",
            "item_fire_damagemax_perlevel" => "每级火焰伤害上限",
            "item_pois_damagemax_perlevel" => "每级毒素伤害上限",
            "item_magic_damagemax_perlevel" => "每级魔法伤害上限",
            "item_fire_damagemin_bytime" => "火焰持续伤害下限",
            "item_magicarrow" => "魔法箭",
            "item_explosivearrow" => "爆炸箭",
            "item_throw_maxdamage" => "投掷最大伤害",
            "item_throw_mindamage" => "投掷最小伤害",
            "item_kickdamage" => "踢击伤害",
            "secondary_mindamage" => "附加最小伤害",
            "secondary_maxdamage" => "附加最大伤害",
            "item_normaldamage" => "普通攻击伤害",
            // 吸收/抗性
            "item_absorb_fire_percent" => "火焰吸收%",
            "item_absorb_ltng_percent" => "闪电吸收%",
            "item_absorb_cold_percent" => "冰冷吸收%",
            "item_absorb_pois_percent" => "毒素吸收%",
            "item_absorb_magic_percent" => "魔法吸收%",
            "item_absorb_fire_bytime" => "火焰吸收(持续)",
            "item_absorb_ltng_bytime" => "闪电吸收(持续)",
            "item_absorb_cold_bytime" => "冰冷吸收(持续)",
            "item_resist_cold_bytime" => "冰抗(持续)",
            "item_resist_ltng_bytime" => "电抗(持续)",
            "item_resist_fire_bytime" => "火抗(持续)",
            "item_absorb_cold_perlevel" => "每级冰冷吸收",
            "item_absorb_ltng_perlevel" => "每级闪电吸收",
            "item_absorb_fire_perlevel" => "每级火焰吸收",
            "item_resist_cold_perlevel" => "每级冰抗",
            "maxmagicresist" => "最大魔抗",
            "magicresist" => "魔法抗性",
            "damageresist" => "伤害抗性",
            // 攻击效果
            "item_crushingblow_bytime" => "压碎打击(持续)",
            "item_fractionaltargetac" => "百分比降低防御",
            "item_reanimate" => "复活怪物",
            "item_skillonlevelup" => "升级时获得技能",
            "item_extra_stack" => "额外叠加",
            // 防御/耐久
            "item_armor_bytime" => "防御(持续)",
            "item_armor_perlevel" => "每级防御",
            "item_armorpercent_perlevel" => "每级防御%",
            "item_maxdurability_percent" => "最大耐久%",
            "item_durability_bytime" => "耐久(持续)",
            "armorclass_vs_missile" => "远程防御",
            "armorclass_vs_hth" => "近战防御",
            // 生命/法力/恢复
            "item_hp_bytime" => "生命(持续)",
            "item_mana_bytime" => "法力(持续)",
            "item_stamina_bytime" => "耐力(持续)",
            "item_energy_bytime" => "能量(持续)",
            "item_dexterity_bytime" => "敏捷(持续)",
            "item_stamina_perlevel" => "每级耐力",
            "item_healafterkill" => "杀敌回血",
            "item_healafterdemonkill" => "杀恶魔回血",
            "item_mana_after_kill" => "杀敌回蓝",
            "item_regenstamina_bytime" => "耐力恢复(持续)",
            "item_regenstamina_perlevel" => "每级耐力恢复",
            "stamdrainmindam" => "耐力损耗最小",
            "stamdrainmaxdam" => "耐力损耗最大",
            "manadrainmindam" => "法力损耗最小",
            "manadrainmaxdam" => "法力损耗最大",
            "lifedrainmindam" => "生命损耗最小",
            "lifedrainmaxdam" => "生命损耗最大",
            // 命中/伤害/MF
            "item_tohit_perlevel" => "每级命中",
            "item_tohitpercent_bytime" => "命中(持续)",
            "item_tohit_vs_monster" => "对怪物命中",
            "item_tohit_percent_vs_monster" => "对怪物命中%",
            "item_ac_vs_monster" => "对怪物防御",
            "item_damage_vs_monster" => "对怪物伤害",
            "item_demondamage_percent" => "对恶魔伤害%",
            "item_undeaddamage_percent" => "对不死伤害%",
            "item_damage_undead_perlevel" => "每级对不死伤害",
            "item_damage_undead_bytime" => "对不死伤害(持续)",
            "item_tohit_demon_perlevel" => "每级对恶魔命中",
            "item_tohit_demon_bytime" => "对恶魔命中(持续)",
            "item_tohit_undead_perlevel" => "每级对不死命中",
            "item_attackertakesdamage" => "攻击反伤",
            "item_attackertakeslightdamage" => "攻击者闪电反伤",
            "item_attackertakescolddamage" => "攻击者冰霜反伤",
            "item_find_magic_bytime" => "MF(持续)",
            "item_find_magic_perlevel" => "每级MF",
            "item_find_gems_bytime" => "宝石掉落(持续)",
            // 技能/施法
            "item_elemskill" => "元素技能",
            "item_slash_damage" => "挥砍伤害",
            "item_slash_damage_percent" => "挥砍伤害%",
            "item_thrust_damage" => "穿刺伤害",
            "item_thrust_damage_percent" => "穿刺伤害%",
            "item_crush_damage" => "粉碎伤害",
            "item_crush_damage_percent" => "粉碎伤害%",
            "item_absorb_slash" => "挥砍吸收",
            "item_absorb_thrust" => "穿刺吸收",
            "item_absorb_crush" => "粉碎吸收",
            "skill_bypass_undead" => "无视不死抗性",
            "skill_bypass_demons" => "无视恶魔抗性",
            "skill_pierce" => "穿透",
            "skill_armor_percent" => "技能护甲%",
            "skill_staminapercent" => "技能耐力%",
            "skill_missile_damage_scale" => "远程伤害缩放",
            "skill_chillingarmor" => "冰封护甲",
            "skill_handofathena" => "雅典娜之手",
            "skill_conviction" => "审判灵气",
            "skill_concentration" => "专注灵气",
            "skill_channeling_tick" => "引导技能间隔",
            "skill_cooldown" => "冷却时间",
            "attackrate" => "攻击速率",
            "attack_vs_montype" => "对特定类型攻击",
            "damage_vs_montype" => "对特定类型伤害",
            "damage_framerate" => "伤害帧率",
            // 注灵体系 (coi_inf_*) — 官方 D2RMM.mpq item-modifiers.json 翻译
            "coi_inf_t1_count" => "T1初阶已注灵次数",
            "coi_inf_t1_gate" => "T1初阶注灵上限",
            "coi_inf_t2_count" => "T2中阶已注灵次数",
            "coi_inf_t2_gate" => "T2中阶注灵上限",
            "coi_inf_t3_count" => "T3高阶已注灵次数",
            "coi_inf_t3_gate" => "T3高阶注灵上限",
            "coi_inf_gate_init" => "通脉状态",
            // 聚字碑体系 (coi_jzb_*)
            "coi_jzb_lin" => "灵印存量", "coi_jzb_xfu" => "仙符存量", "coi_jzb_lsh" => "灵石存量", "coi_jzb_lyd" => "灵蕴点存量",
            "coi_jzb_jlf" => "聚灵符存量", "coi_jzb_rly" => "融灵药剂存量", "coi_jzb_nls" => "凝灵砂存量", "coi_jzb_lck" => "幸运符存量",
            "coi_jzb_cly" => "测灵玉存量", "coi_jzb_qlf" => "启灵符存量", "coi_jzb_cll" => "测灵令存量", "coi_jzb_lgs" => "灵根神石存量",
            "coi_jzb_lgy" => "灵根源液存量", "coi_jzb_uni" => "暗金精华存量",
            // 灵根体系 (coi_root_*)
            "coi_root_gold" => "金灵根等级", "coi_root_wood" => "木灵根等级", "coi_root_water" => "水灵根等级", "coi_root_fire" => "火灵根等级",
            "coi_root_earth" => "土灵根等级", "coi_root_light" => "明灵根等级", "coi_root_dark" => "暗灵根等级",
            "coi_mfp" => "提升150%魔法寻获几率",
            // 通用杂项
            "item_timeduration" => "持续时间",
            "item_lightcolor" => "光照颜色",
            "item_req_amulet" => "需求-护身符",
            "item_req_ring" => "需求-戒指",
            "item_poisonlengthresist" => "毒素持续时间缩短",
            "firemindam" => "最小火焰伤害",
            "firemaxdam" => "最大火焰伤害",
            "lightmindam" => "最小闪电伤害",
            "lightmaxdam" => "最大闪电伤害",
            "coldmindam" => "最小冰霜伤害",
            "coldmaxdam" => "最大冰霜伤害",
            "poisonmindam" => "最小毒素伤害",
            "poisonmaxdam" => "最大毒素伤害",
            "progressive_damage" => "成长-伤害",
            "progressive_tohit" => "成长-命中",
            "progressive_steal" => "成长-偷取",
            "progressive_fire" => "成长-火焰",
            "progressive_cold" => "成长-冰霜",
            "progressive_lightning" => "成长-闪电",
            "progressive_other" => "成长-其他",
            "statpts" => "属性点",
            "goldbank" => "仓库金币",
            "lasthitreactframe" => "受击反应帧",
            "stunlength" => "眩晕时长",
            "firelength" => "燃烧时长",
            "poisoncount" => "毒素叠加",
            "pierce_idx" => "穿透索引",
            "curse_resistance" => "诅咒抗性",
            "questitemdifficulty" => "任务物品难度",
            "hitpoints" => "生命值",
            "progressive_throw_dmg" => "成长-投掷伤害",
            "progressive_mana" => "成长-法力",
            "progressive_hp" => "成长-生命",
            "life_resist" => "生命抗性",
            "explosion_resist" => "爆炸抗性",
            "thorns_percent" => "荆棘伤害%",
            "missile_thorns_percent" => "远程荆棘%",
            _ => return None,
        })
    }

    /// 格式化多组 stat（去重）
    pub fn format_stats(
        main_stats: &[ItemStat],
        runeword_stats: &[ItemStat],
        language: &str,
        _conn: Option<&Connection>,
        _profile_id: i64,
    ) -> Vec<String> {
        let mut out = Vec::new();
        let all: Vec<&ItemStat> = main_stats.iter().chain(runeword_stats.iter()).collect();

        // Python: group resistances — if 2+ resists have same value, combine into one line
        let resist_ids: [u16; 4] = [39, 41, 43, 45]; // fire, lightning, cold, poison
        let resist_labels = ["火", "电", "冰", "毒"];
        let mut resist_map: Vec<(i64, usize)> = Vec::new(); // (value, index)
        for (i, &rid) in resist_ids.iter().enumerate() {
            if let Some(s) = all.iter().find(|st| st.id == rid && st.value != 0) {
                resist_map.push((s.value, i));
            }
        }
        // Also check "by time" variants 285-287, 284 (fire, lightning, cold, poison by-time)
        // 2026-07-31 fix: 旧映射 [151,150,149,148] 是错的 — 148=冰冷吸收%, 149=冰冷吸收,
        // 150=减速, 151=灵气 (item_aura, 如眼光的冥想灵气)。by-time 抗性真实 ID 是 285-287,284。
        let resist_bt_ids: [u16; 4] = [285, 286, 287, 284]; // fire, lightning, cold, poison by-time
        for (i, &rid) in resist_bt_ids.iter().enumerate() {
            if let Some(s) = all.iter().find(|st| st.id == rid && st.value != 0)
                && !resist_map.iter().any(|(_, idx)| *idx == i) {
                    resist_map.push((s.value, i));
                }
        }
        if resist_map.len() >= 2 {
            let all_same = resist_map.iter().all(|(v, _)| *v == resist_map[0].0);
            if all_same {
                let line = match language {
                    "enUS" => format!("All Resistances +{}%", resist_map[0].0),
                    _ => format!("全抗 +{}%", resist_map[0].0),
                };
                out.push(line);
            } else {
                let parts: Vec<String> = resist_map.iter().map(|(v, i)| format!("{}+{}%", resist_labels[*i], v)).collect();
                let line = match language {
                    "enUS" => format!("Resistances: {}", parts.join("/")),
                    _ => format!("抗性: {}", parts.join("/")),
                };
                out.push(line);
            }
        } else if resist_map.len() == 1 {
            let (v, i) = resist_map[0];
            let line = match language {
                "enUS" => format!("{} Resist +{}%", match i { 0 => "Fire", 1 => "Lightning", 2 => "Cold", _ => "Poison" }, v),
                _ => format!("{}抗 +{}%", resist_labels[i], v),
            };
            out.push(line);
        }

        let mut i = 0;
        while i < all.len() {
            let stat = all[i];
            // Skip zero-value stats
            if stat.value == 0 {
                i += 1;
                continue;
            }
            // Skip individual resists that were already grouped above
            if resist_ids.contains(&stat.id) || resist_bt_ids.contains(&stat.id) {
                i += 1;
                continue;
            }
            // Element damage merges — consecutive stat IDs (NP-corrected)
            // Lightning: 50(min)+51(max)
            if stat.id == 50 && i + 1 < all.len() && (all[i+1].id == 51 || all[i+1].id == 50) {
                let min_v = stat.value; let max_v = all[i+1].value;
                let line = match language {
                    "enUS" => format!("Adds {}-{} Lightning Damage", min_v, max_v),
                    _ => format!("附加 {}-{} 闪电伤害", min_v, max_v),
                };
                if !out.contains(&line) { out.push(line); }
                i += 2; continue;
            }
            // Cold: 54(min)+55(max)+56(length)
            if stat.id == 54 && i + 2 < all.len() {
                let min_v = stat.value; let max_v = all[i+1].value; let len = all[i+2].value;
                let secs = len as f64 / 25.0;
                let line = match language {
                    "enUS" => format!("Adds {}-{} Cold Damage, {:.1} sec", min_v, max_v, secs),
                    _ => format!("附加 {}-{} 冰霜伤害, 持续 {:.0} 秒", min_v, max_v, secs),
                };
                if !out.contains(&line) { out.push(line); }
                i += 3; continue;
            }
            // Poison: 57(min)+58(max)+59(length)
            // Display = raw × length / 256
            if stat.id == 57 && i + 2 < all.len() {
                let min_v = stat.value; let max_v = all[i+1].value; let len = all[i+2].value;
                let min_d = (min_v as f64 * len as f64 / 256.0).round() as i64;
                let max_d = (max_v as f64 * len as f64 / 256.0).round() as i64;
                let secs = len as f64 / 25.0;
                let (dmg, sec_str) = if min_d == max_d {
                    (format!("{}", min_d), format!("{:.0}", secs))
                } else {
                    (format!("{}-{}", min_d, max_d), format!("{:.0}", secs))
                };
                let line = match language {
                    "enUS" => format!("+{} Poison Damage, {} sec", dmg, sec_str),
                    _ => format!("+{} 毒素伤害, 时效 {} 秒", dmg, sec_str),
                };
                if !out.contains(&line) { out.push(line); }
                i += 3; continue;
            }
            // Magic: 52(min)+53(max)
            if stat.id == 52 && i + 1 < all.len() && (all[i+1].id == 53 || all[i+1].id == 52) {
                let min_v = stat.value; let max_v = all[i+1].value;
                let line = match language {
                    "enUS" => format!("Adds {}-{} Magic Damage", min_v, max_v),
                    _ => format!("附加 {}-{} 魔法伤害", min_v, max_v),
                };
                if !out.contains(&line) { out.push(line); }
                i += 2; continue;
            }
            // 通用 stat 格式化（力量、敏捷、生命、MF 等）
            if let Some(line) = Self::format_stat(stat, language, _conn, _profile_id)
                && !out.contains(&line) { out.push(line); }
            i += 1;
            continue;
        }
        out
    }

    /// 已知物品的特殊 stat 行（硬编码兜底）
    pub fn known_stats(
        code: &str,
        unique_id: Option<u16>,
        set_id: Option<u16>,
        runeword_id: Option<u16>,
    ) -> Vec<&'static str> {
        // Insight runeword
        if code == "7s8" && runeword_id == Some(88) {
            return vec![
                "+5 全属性", "+35% 快速施法", "冥思灵气",
                "+增强伤害 / 攻击准确率", "+魔法物品获取率",
            ];
        }
        match unique_id {
            Some(248) => vec!["+2 全技能", "10% 伤害减少", "50% 更佳机会取得魔法装备"],
            Some(275) => vec!["无法冰冻", "20% 冰吸收", "+敏捷 / 法力 / 准确率"],
            Some(274) => vec!["15% 火焰吸收", "+生命 / 体力", "魔法伤害降低 / 打钱"],
            Some(373) => vec!["+1 全技能", "+20% 快速施法", "+5% 最大法力"],
            Some(105) => vec!["+20% 快速施法", "法力恢复提高 25%", "+1 火系技能"],
            Some(253) => vec!["35% 伤害减少", "+30 力量", "+25% 格挡几率"],
            Some(413) => vec!["30% 高速跑步", "20% 快速打击恢复", "+能量 / 敏捷"],
            Some(409) => vec!["+2 术士技能页", "被击中时诅咒触发", "减伤 / 每级生命"],
            _ => match set_id {
                Some(136) => vec!["+防御强化", "元素抗性", "技能页加成"],
                _ => Vec::new(),
            },
        }
    }

    fn format_stat_zh(stat: &ItemStat) -> Option<String> {
        let v = stat.value;
        let dv = stat.display_value();
        match stat.id {
            // 基础属性 (use display_value for save_add correction)
            0 => Some(Self::prefix_stat("力量", dv)),
            1 => Some(Self::prefix_stat("能量", dv)),
            2 => Some(Self::prefix_stat("敏捷", dv)),
            3 => Some(Self::prefix_stat("体力", dv)),
            6 => Some(Self::prefix_stat("生命", v)),
            7 => Some(Self::prefix_stat("生命", dv)),
            8 => Some(Self::prefix_stat("法力", v)),
            9 => Some(Self::prefix_stat("法力", dv)),
            11 => Some(Self::prefix_stat("耐力", v)),
            // 防御/伤害
            16 => Some(format!("+{}% 防御强化", v)),
            17 => Some(format!("最大伤害 +{}%", v)),
            18 => Some(format!("最小伤害 +{}%", v)),
            25 => Some(format!("+{}% 增强伤害", v)),
            26 => Some(Self::prefix_stat("法力恢复", v)),
            27 => Some(format!("法力恢复提高 {}%", v)),
            204 => Some(format!("+{} 技能充能", v)),
            19 => Some(format!("+{}% 准确率", v)),
            24 => Some(format!("+{} 准确率", v)),
            21 => Some(format!("最小伤害 +{}", v)),
            22 => Some(format!("最大伤害 +{}", v)),
            31 => Some(Self::prefix_stat("防御", v)),
            34 => Some(format!("伤害减少 {}", v)),
            35 => Some(format!("魔法伤害降低 {}", v)),
            36 => Some(format!("物理伤害减轻 {}", v)),
            // 元素抗性（基础）
            39 => Some(format!("抗火 +{}%", v)),
            41 => Some(format!("抗电 +{}%", v)),
            43 => Some(format!("抗冰 +{}%", v)),
            45 => Some(format!("抗毒 +{}%", v)),
            // 抗性吸收 (vanilla D2R 含义)
            // 魔法吸收 (Magic Absorb)
            147 => Some(format!("魔法吸收 +{}%", v)),
            148 => Some(format!("冰冷吸收 {}%", v)),
            // 特殊状态
            108 => Some("怪物安息".to_string()),
            117 => Some("阻止怪物治疗".to_string()),
            118 => Some("冰冻时间减半".to_string()),
            // 耐久/生命/法力
            72 => Some(format!("耐久度 {}", v)),
            73 => Some(format!("最大耐久度 {}", v)),
            74 => Some(Self::prefix_stat("回复生命", v)),
            76 => Some(format!("+{}% 最大生命", v)),
            77 => Some(format!("+{}% 最大法力", v)),
            78 => Some(format!("攻击反伤 {}", v)),
            79 => Some(format!("{}% 额外金币", v)),
            80 | 161 | 165 => Some(format!("{}% 更佳机会取得魔法装备", v)),
            86 => Some(Self::prefix_stat("杀死回血", v)),
            87 => Some(Self::prefix_stat("杀死回蓝", v)),
            88 | 137 => Some(format!("{}% 生命偷取", v)),
            89 => Some(format!("光照 +{}", v)),
            // 攻速/FHR/FCR
            93 => Some(format!("{}% 攻击速度", v)),
            94 | 99 => Some(format!("{}% 快速打击恢复", v)),
            95 | 96 => Some(format!("{}% 高速跑步/行走", v)),
            97 | 105 | 163 => Some(format!("{}% 快速施法", v)),
            102 => Some(format!("{}% 快速格挡恢复", v)),
            83 => Some(format!("+{} 职业技能", v)),
            // 技能相关
            107 => Some(format!("+{} 单技能加成", v)),
            116 | 127 => Some(Self::prefix_stat("所有技能", v)),
            188 => Some(format!("{} +{}", skill_tab_name(stat.param), v)),
            // 伤害附加
            111 => Some(Self::prefix_stat("物理伤害", v)),
            112 => Some(Self::prefix_stat("命中", v)),
            114 => Some(format!("受损失的 {}% 伤害转换到法力", v)),
            120 => Some(Self::prefix_stat("法力恢复", v)),
            // 特殊攻击效果
            136 | 143 => Some(format!("{}% 压碎打击", v)),
            138 => Some(format!("杀敌后恢复法力 +{}", v)),
            141 => Some(format!("{}% 致命攻击", v)),
            142 => Some(format!("火焰吸收 {}%", v)),
            144 => Some(format!("冻结目标 +{}", v)),
            150 => Some(format!("减慢目标 {}%", v)),
            // 灵气 (stat 151)
            151 => {
                let aura = skill_name(stat.param).unwrap_or("Aura");
                Some(format!("等级 {} {}灵气", stat.value, aura))
            }
            // 通用辅助
            162 | 166 => Some(format!("{}% 额外金币", v)),
            252 => Some(format!("{} 秒回复耐久度", v)),
            253 => Some("自动回复数量".to_string()),
            195 => Some(format!("+{}% 防御强化", v)),
            // 元素伤害（部分需要 ID 相邻合并 — 见 format_stats）
            50 => None,  // merged with next 50
            51 => None,  // merged with 50 via NP
            196 => Some(format!("最小冰霜伤害 +{}", v)),
            197 => Some(format!("最大冰霜伤害 +{}", v)),
            199 => Some(format!("最小火焰伤害 +{}", v)),
            200 => Some(format!("最大火焰伤害 +{}", v)),
            201 => Some(format!("最小闪电伤害 +{}", v)),
            202 => Some(format!("最大闪电伤害 +{}", v)),
            203 => Some(format!("最小毒素伤害 +{}", v)),
            // 每级成长
            225 => Some(format!("每级 +{} 准确率", v)),
            230 => Some(format!("每级 +{}% 冰抗", v)),
            259 => Some(format!("攻击反伤 {}", v)),
            261 => Some(format!("{}% 使怪物逃跑", v)),
            // 生命回复 / 恢复
            119 => Some(format!("回复生命 +{}", v)),
            // 使目标致盲
            145 => Some("击中使目标致盲".to_string()),
            // 无法冰冻 (stat 152, 153, 167)
            152 | 153 | 167 => Some("无法冰冻".to_string()),
            // 魔法/物理抗性 (stat 156, 157)
            156 => Some(format!("魔法抗性 +{}%", v)),
            157 => Some(format!("物理抗性 +{}%", v)),
            // 需求减少
            164 => Some(format!("需求 -{}%", v)),
            // 魔改 mod stat: d2emu 体系
            329 => Some(format!("火焰掌握 +{}", v)),
            _ => None,
        }
    }
    /// 带符号前缀的属性值: "+N 力量" / "-N 力量"
    fn prefix_stat(label: &str, v: i64) -> String {
        if v >= 0 { format!("+{} {}", v, label) } else { format!("{} {}", v, label) }
    }

    // ── Private: 英文 stat 格式化 ──

    fn format_stat_en(stat: &ItemStat) -> Option<String> {
        let v = stat.value;
        let dv = stat.display_value();
        match stat.id {
            // Base stats
            0 => Some(Self::prefix_stat_en("Strength", dv)),
            1 => Some(Self::prefix_stat_en("Energy", dv)),
            2 => Some(Self::prefix_stat_en("Dexterity", dv)),
            3 => Some(Self::prefix_stat_en("Vitality", dv)),
            6 => Some(Self::prefix_stat_en("Life", v)),
            7 => Some(Self::prefix_stat_en("Life", dv)),
            8 => Some(Self::prefix_stat_en("Mana", v)),
            9 => Some(Self::prefix_stat_en("Mana", dv)),
            11 => Some(Self::prefix_stat_en("Stamina", v)),
            // Def/Damage
            16 => Some(format!("+{}% Enhanced Defense", v)),
            17 | 18 | 25 => Some(format!("+{}% Enhanced Damage", v)),
            19 => Some(format!("+{} Attack Rating", v)),
            22 => Some(format!("+{} Max Damage", v)),
            24 => Some(format!("+{} to Attack Rating", v)),
            26 => Some(format!("+{}% Mana Regeneration", v)),
            21 => Some(format!("+{} Min Damage", v)),
            31 => Some(Self::prefix_stat_en("Defense", v)),
            34 => Some(format!("Damage Reduced by {}", v)),
            35 => Some(format!("Magic Damage Reduced by {}", v)),
            36 => Some(format!("Physical Damage Reduced by {}", v)),
            // Resistances
            39 => Some(format!("Fire Resist +{}%", v)),
            41 => Some(format!("Lightning Resist +{}%", v)),
            43 => Some(format!("Cold Resist +{}%", v)),
            45 => Some(format!("Poison Resist +{}%", v)),
            142 => Some(format!("Fire Absorb {}%", v)),
            147 => Some(format!("Magic Absorb {}%", v)),
            148 => Some(format!("Cold Absorb {}%", v)),
            152 | 153 | 167 => Some("Cannot Be Frozen".to_string()),
            156 => Some(format!("Magic Resist +{}%", v)),
            157 => Some(format!("Physical Resist +{}%", v)),
            // Misc stats
            72 => Some(format!("Durability {}", v)),
            73 => Some(format!("Max Durability {}", v)),
            74 => Some(Self::prefix_stat_en("Life Regenerated", v)),
            76 => Some(format!("+{}% Maximum Life", v)),
            77 => Some(format!("+{}% Maximum Mana", v)),
            78 => Some(format!("Attacker Takes Damage {}", v)),
            79 => Some(format!("{}% Extra Gold", v)),
            80 | 161 | 165 => Some(format!("{}% Better Chance of Getting Magic Items", v)),
            83 => Some(format!("+{} to Class Skills", v)),
            86 => Some(format!("Rep Life +{}", v)),
            87 => Some(format!("Rep Mana +{}", v)),
            88 | 137 => Some(format!("{}% Life Steal", v)),
            89 => Some(format!("Light Radius +{}", v)),
            93 => Some(format!("{}% Increased Attack Speed", v)),
            94 | 99 => Some(format!("{}% Faster Hit Recovery", v)),
            95 | 96 => Some(format!("{}% Faster Run/Walk", v)),
            97 | 105 | 163 => Some(format!("{}% Faster Cast Rate", v)),
            102 => Some(format!("{}% Faster Block Rate", v)),
            // Skills
            107 => Some(format!("+{} to Single Skill", v)),
            108 => Some("Slain Monsters Rest in Peace".to_string()),
            117 => Some("Prevent Monster Heal".to_string()),
            118 => Some("Half Freeze Duration".to_string()),
            188 => Some(format!("{} +{}", skill_tab_name_en(stat.param), v)),
            // Damage bonuses
            111 => Some(Self::prefix_stat_en("Physical Damage", v)),
            112 => Some(Self::prefix_stat_en("Attack Rating", v)),
            114 => Some(format!("{}% Damage Taken Gained as Mana", v)),
            119 => Some(format!("Replenish Life +{}", v)),
            120 => Some(format!("Regenerate Mana +{}", v)),
            // Special attack effects
            136 | 143 => Some(format!("{}% Crushing Blow", v)),
            138 => Some(format!("Mana After Each Kill +{}", v)),
            141 => Some(format!("{}% Deadly Strike", v)),
            144 => Some(format!("Freezes Target +{}", v)),
            145 => Some("Hit Blinds Target".to_string()),
            150 => Some(format!("Slows Target by {}%", v)),
            151 => {
                let aura = skill_name(stat.param).unwrap_or("Aura");
                Some(format!("Level {} {} Aura", stat.value, aura))
            }
            // Misc
            162 | 166 => Some(format!("{}% Extra Gold", v)),
            164 => Some(format!("Requirements -{}%", v)),
            193 | 194 => Some(format!("Socketed {}", v)),
            195 => Some(format!("+{}% Enhanced Defense", v)),
            // Elemental damage
            50 => None,
            51 => None,  // merged with 50 via NP
            196 => Some(format!("Min Cold Damage +{}", v)),
            197 => Some(format!("Max Cold Damage +{}", v)),
            199 => Some(format!("Min Fire Damage +{}", v)),
            200 => Some(format!("Max Fire Damage +{}", v)),
            201 => Some(format!("Min Lightning Damage +{}", v)),
            202 => Some(format!("Max Lightning Damage +{}", v)),
            203 => Some(format!("Min Poison Damage +{}", v)),
            // Per-level
            225 => Some(format!("+{} Attack Rating Per Level", v)),
            // Misc
            252 => Some(format!("Repair 1 Durability in {} Seconds", v)),
            253 => Some("Replenishes Quantity".to_string()),
            // Mod stats
            // Per-level
            230 => Some(format!("+{}% Cold Resist Per Level", v)),
            // On-strike
            259 => Some(format!("Attacker Takes Damage {}", v)),
            261 => Some(format!("{}% Hit Causes Monster to Flee", v)),
            329 => Some(format!("Fire Mastery +{}", v)),
            _ => None,
        }
    }
    fn prefix_stat_en(label: &str, v: i64) -> String {
        if v >= 0 { format!("+{} {}", v, label) } else { format!("{} {}", v, label) }
    }

    // ── Private: 标签文案 ──

    fn slot_label(_language: &str) -> &'static str {
        match _language {
            "zhCN" | "zhTW" => "槽位",
            _ => "Slot",
        }
    }

    fn slot_name<'a>(slot: &'a str, language: &str) -> &'a str {
        if language.starts_with("zh") {
            match slot {
                "helm" => "头部", "amulet" => "护身符",
                "ring_l" => "左戒指", "ring_r" => "右戒指",
                "armor" => "护甲",
                "weapon_main" => "主武器", "shield_main" => "主副手",
                "weapon_alt" => "副武器", "shield_alt" => "副副手",
                "gloves" => "手套", "boots" => "靴子", "belt" => "腰带",
                _ => slot,
            }
        } else {
            match slot {
                "helm" => "Helm", "amulet" => "Amulet",
                "ring_l" => "Left Ring", "ring_r" => "Right Ring",
                "armor" => "Armor",
                "weapon_main" => "Main Weapon", "shield_main" => "Main Shield",
                "weapon_alt" => "Alt Weapon", "shield_alt" => "Alt Shield",
                "gloves" => "Gloves", "boots" => "Boots", "belt" => "Belt",
                _ => slot,
            }
        }
    }

    fn qty_label(_language: &str) -> &'static str {
        match _language {
            "zhCN" | "zhTW" => "数量",
            _ => "Quantity",
        }
    }

    pub(crate) fn meta_line(key: &str, value: &str, language: &str) -> String {
        let localized = match key {
            "Code" if language.starts_with("zh") => "代码",
            "Type" if language.starts_with("zh") => "类型",
            "Quality" if language.starts_with("zh") => "品质",
            "ItemLevel" if language.starts_with("zh") => "物品等级",
            _ => key,
        };
        format!("{}: {}", localized, value)
    }
}

// ── Module-level helpers (not tied to TooltipFormatter struct) ──

/// Convert quality byte to display name (localized).
pub fn quality_name(quality_byte: u8, language: &str) -> Option<&'static str> {
    let is_zh = language.starts_with("zh");
    match quality_byte {
        1 => Some(if is_zh { "劣质" } else { "low" }),
        2 => Some(if is_zh { "普通" } else { "normal" }),
        3 => Some(if is_zh { "超强" } else { "superior" }),
        4 => Some(if is_zh { "魔法" } else { "magic" }),
        5 => Some(if is_zh { "套装" } else { "set" }),
        6 => Some(if is_zh { "稀有" } else { "rare" }),
        7 => Some(if is_zh { "暗金" } else { "unique" }),
        8 => Some(if is_zh { "手工" } else { "crafted" }),
        _ => None,
    }
}

/// Localize item type display name.
/// Queries `localized_string` with namespace `item_types`. Falls back to the
/// English kind string when DB or entry is unavailable.
pub fn type_name(kind: &str, language: &str, conn: Option<&Connection>, profile_id: i64) -> String {
    if language == "enUS" {
        return kind.to_string();
    }
    if let Some(conn) = conn {
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT text_value FROM localized_string
             WHERE profile_id = ?1 AND namespace = 'item_types' AND string_key = ?2 AND language = ?3
             LIMIT 1",
        ) {
            let result: Option<String> = stmt.query_row(
                rusqlite::params![profile_id, kind.to_lowercase(), language],
                |row| row.get(0),
            )
            .ok();
            if let Some(name) = result {
                return name;
            }
        }

        if let Ok(vp_id) = conn.query_row(
            "SELECT id FROM resource_profile WHERE profile_key LIKE 'vanilla:%' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
            && let Ok(mut stmt) = conn.prepare_cached(
                "SELECT text_value FROM localized_string
                 WHERE profile_id = ?1 AND namespace = 'item_types' AND string_key = ?2 AND language = ?3
                 LIMIT 1",
            ) {
                let result: Option<String> = stmt.query_row(
                    rusqlite::params![vp_id, kind.to_lowercase(), language],
                    |row| row.get(0),
                )
                .ok();
                if let Some(name) = result {
                    return name;
                }
            }
    }
    // Hardcoded Chinese fallback for item types (used when DB has no translation)
    if language.starts_with("zh") {
        return match kind.to_lowercase().as_str() {
            "rune" => "符文",
            "gem" => "宝石",
            "key" => "钥匙",
            "essence" => "精华",
            "shard" => "碎片",
            "charm" => "护符",
            "jewelry" => "珠宝",
            "armor" => "护甲",
            "weapon" => "武器",
            "shield" => "盾牌",
            "token" => "徽章",
            "misc" => "杂项",
            _ => kind,
        }.to_string();
    }
    kind.to_string()
}

/// Resolve skill name from skill ID.
pub fn skill_name(skill_id: u32) -> Option<&'static str> {
    match skill_id {
        120 => Some("Meditation"),
        _ => None,
    }
}

/// Resolve skill tab name (Chinese).
pub fn skill_tab_name(param: u32) -> String {
    match param {
        21 => "术士技能页".to_string(),
        _ => match param & 0x7 {
            0 => "战斗技能".to_string(),
            1 => "被动和魔法技能".to_string(),
            2 => "弓和十字弓技能".to_string(),
            v => format!("技能页 {}", v),
        },
    }
}

/// Resolve skill tab name (English).
pub fn skill_tab_name_en(param: u32) -> String {
    match param {
        21 => "Sorceress Skill Tab".to_string(),
        _ => match param & 0x7 {
            0 => "Combat Skills".to_string(),
            1 => "Passive and Magic Skills".to_string(),
            2 => "Bow and Crossbow Skills".to_string(),
            v => format!("Skill Tab {}", v),
        },
    }
}

/// Slot display label (Chinese) — retained for backward compat.
pub fn display_slot_label(slot: &str) -> &str {
    match slot {
        "helm" => "头部", "amulet" => "护身符",
        "ring_l" => "左戒指", "ring_r" => "右戒指",
        "armor" => "护甲",
        "weapon_main" => "主武器", "shield_main" => "主副手",
        "weapon_alt" => "副武器", "shield_alt" => "副副手",
        "gloves" => "手套", "boots" => "靴子", "belt" => "腰带",
        _ => slot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_stat_strength_zh() {
        let stat = ItemStat { id: 0, param: 0, value: 10, ..Default::default() };
        let line = TooltipFormatter::format_stat(&stat, "zhCN", None, 0);
        assert_eq!(line, Some("+10 力量".to_string()));
    }

    #[test]
    fn test_format_stat_strength_en() {
        let stat = ItemStat { id: 0, param: 0, value: 10, ..Default::default() };
        let line = TooltipFormatter::format_stat(&stat, "enUS", None, 0);
        assert_eq!(line, Some("+10 Strength".to_string()));
    }

    #[test]
    fn test_format_stat_unknown_returns_none() {
        let stat = ItemStat { id: 999, param: 0, value: 10, ..Default::default() };
        let line = TooltipFormatter::format_stat(&stat, "zhCN", None, 0);
        assert_eq!(line, None);
    }

    #[test]
    fn test_format_stats_dedup() {
        let s1 = ItemStat { id: 0, param: 0, value: 10, ..Default::default() };
        let s2 = ItemStat { id: 0, param: 0, value: 10, ..Default::default() }; // dup
        let lines = TooltipFormatter::format_stats(&[s1, s2], &[], "zhCN", None, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "+10 力量");
    }

    #[test]
    fn test_quality_name() {
        assert_eq!(quality_name(7, "enUS"), Some("unique"));
        assert_eq!(quality_name(2, "zhCN"), Some("普通"));
        assert_eq!(quality_name(99, "enUS"), None);
    }

    #[test]
    fn test_stash_tooltip_zh() {
        let lines = TooltipFormatter::stash_tooltip("艾尔", "El Rune", "r01", "rune", Some(2), 1, "zhCN", &[]);
        assert!(lines[0].contains("艾尔"));
        assert!(lines.iter().any(|l| l.contains("rune")));
    }

    #[test]
    fn test_stash_tooltip_with_quantity() {
        let lines = TooltipFormatter::stash_tooltip("El Rune", "El Rune", "r01", "rune", Some(2), 3, "enUS", &[]);
        assert!(lines.iter().any(|l| l.contains("3")));
    }

    #[test]
    fn test_equipment_tooltip_basic() {
        // Without a DB connection, type_name falls back to the raw kind string.
        // The first line format is: "Type: <type_name> [Quality: <q>]"
        let lines = TooltipFormatter::equipment_tooltip(
            "helm", "cap", 2, Some(85),
            Some("Cap"), Some("帽子"),
            None, None, None,
            &[], &[],
            0, 0, 0,
            "zhCN",
            None, 0,
            true,
        );
        // Without DB, type_name("helm", "zhCN", None, 0) returns "helm" (English fallback)
        assert!(lines[0].contains("头部"), "line 0 is slot name, got: {:?}", lines[0]);
        assert!(lines[1].contains("帽子"), "line 1 is chinese name, got: {:?}", lines[1]);
        assert!(lines.iter().any(|l| l.contains("Cap")));
        assert!(lines.iter().any(|l| l.contains("85")));
    }

    #[test]
    fn test_equipment_tooltip_english() {
        let lines = TooltipFormatter::equipment_tooltip(
            "helm", "cap", 2, Some(85),
            Some("Cap"), Some("Cap"),
            None, None, None,
            &[], &[],
            0, 0, 0,
            "enUS",
            None, 0,
            true,
        );
        // enUS always returns the kind string directly: "helm"
        assert!(lines[0].contains("Helm"), "enUS slot name expected, got: {:?}", lines[0]);
    }


    #[test]
    fn test_known_stats_raven_frost() {
        let stats = TooltipFormatter::known_stats("rin", Some(275), None, None);
        assert!(!stats.is_empty());
        assert!(stats.iter().any(|s| s.contains("无法冰冻")));
    }

    #[test]
    fn test_known_stats_unknown_returns_empty() {
        let stats = TooltipFormatter::known_stats("xxx", None, None, None);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_format_stats_lightning_zh() {
        let main_stats = vec![
            crate::protocol::common::ItemStat { id: 50, param: 0, value: 1, ..Default::default() },
            crate::protocol::common::ItemStat { id: 50, param: 0, value: 3, ..Default::default() },
        ];
        let lines = TooltipFormatter::format_stats(&main_stats, &[], "zhCN", None, 0);
        assert!(lines.iter().any(|l| l.contains("1-3") && l.contains("闪电")),
            "Expected '附加 1-3 闪电伤害', got: {:?}", lines);
    }

    #[test]
    fn test_stash_tooltip_with_stats() {
        // Simulate warehouse tooltip with stat_lists from item_json
        let stats = vec![
            crate::protocol::common::ItemStat { id: 0, param: 0, value: 20, ..Default::default() },
            crate::protocol::common::ItemStat { id: 39, param: 0, value: 35, ..Default::default() },
        ];
        let lines = TooltipFormatter::stash_tooltip(
            "精神盾", "Spirit", "pau", "shield", Some(4), 1, "zhCN", &stats,
        );
        assert!(lines[0].contains("精神盾"));
        assert!(lines.iter().any(|l| l.contains("+20 力量")));
        // format_stats groups single resist as "火抗 +35%" (not "抗火 +35%")
        assert!(lines.iter().any(|l| l.contains("火抗") && l.contains("+35%")),
            "Expected fire resist line, got: {:?}", lines);
    }

    #[test]
    fn test_stash_tooltip_with_resistance_combo() {
        // Test resistance grouping in stash tooltip
        let stats = vec![
            crate::protocol::common::ItemStat { id: 39, param: 0, value: 25, ..Default::default() },
            crate::protocol::common::ItemStat { id: 41, param: 0, value: 25, ..Default::default() },
            crate::protocol::common::ItemStat { id: 43, param: 0, value: 25, ..Default::default() },
            crate::protocol::common::ItemStat { id: 45, param: 0, value: 25, ..Default::default() },
        ];
        let lines = TooltipFormatter::stash_tooltip(
            "Test", "test", "rin", "ring", None, 1, "zhCN", &stats,
        );
        assert!(lines.iter().any(|l| l.contains("全抗")),
            "Expected combined resistance line, got: {:?}", lines);
    }

    #[test]
    fn test_stash_tooltip_item_json_round_trip() {
        // Verify that stats stored as JSON (like warehouse item_json) can be
        // deserialized and passed to stash_tooltip correctly
        let stat_lists = vec![
            crate::protocol::common::stat_list::StatList {
                stats: vec![
                    crate::protocol::common::ItemStat { id: 0, param: 0, value: 20, ..Default::default() },
                ],
            },
        ];
        let serialized = serde_json::to_string(&stat_lists).unwrap();
        let deserialized: Vec<crate::protocol::common::stat_list::StatList> =
            serde_json::from_str(&serialized).unwrap();
        let stats: Vec<crate::protocol::common::ItemStat> =
            deserialized.into_iter().flat_map(|sl| sl.stats).collect();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].id, 0);
        assert_eq!(stats[0].value, 20);

        let lines = TooltipFormatter::stash_tooltip(
            "力量指环", "Ring of Strength", "rin", "ring", Some(4), 1, "zhCN", &stats,
        );
        assert!(lines.iter().any(|l| l.contains("+20 力量")));
    }

    #[test]
    fn test_format_stats_lightning_en() {
        let main_stats = vec![
            crate::protocol::common::ItemStat { id: 50, param: 0, value: 1, ..Default::default() },
            crate::protocol::common::ItemStat { id: 50, param: 0, value: 3, ..Default::default() },
        ];
        let lines = TooltipFormatter::format_stats(&main_stats, &[], "enUS", None, 0);
        assert!(lines.iter().any(|l| l.contains("1-3") && l.contains("Lightning")),
            "Expected 'Adds 1-3 Lightning Damage', got: {:?}", lines);
    }
}
