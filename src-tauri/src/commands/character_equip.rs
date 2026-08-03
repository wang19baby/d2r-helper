//! Equipment slot builders using SQLite-backed NameResolver.
//!
//! These functions replace the `HashMap`-based name resolution in `character.rs`
//! with the unified `NameResolver` that queries SQLite definition tables.

use rusqlite::Connection;
use crate::resource::NameResolver;
use crate::commands::character::EquipmentSlotInfo;


/// Normalize slot names for modified items. Mirrors character.rs.
use crate::protocol::d2i::parser::ParsedItem;
use crate::protocol::common::item_location::ItemLocation;

/// Map ItemLocation to equipment slot name.
pub fn location_to_slot(loc: ItemLocation) -> Option<&'static str> {
    Some(match loc {
        ItemLocation::Head => "helm",
        ItemLocation::Neck => "amulet",
        ItemLocation::Torso => "armor",
        ItemLocation::RightHand => "weapon_main",
        ItemLocation::LeftHand => "shield_main",
        ItemLocation::RightFinger => "ring_r",
        ItemLocation::LeftFinger => "ring_l",
        ItemLocation::Waist => "belt",
        ItemLocation::Feet => "boots",
        ItemLocation::Hands => "gloves",
        ItemLocation::Trinket1 => "weapon_alt",
        ItemLocation::Trinket2 => "shield_alt",
        _ => return None,
    })
}

/// Build equipment from `D2SCharacter.equipped` (ParsedItem from d2s::items).
pub fn build_equipment_from_parsed_items(
    items: &[ParsedItem],
    conn: &Connection,
    resolver: &NameResolver,
    tooltip_language: &str,
) -> Vec<EquipmentSlotInfo> {
    use crate::commands::character::EQUIPMENT_SLOTS;

    let mut by_slot: std::collections::HashMap<&str, &ParsedItem> = std::collections::HashMap::new();
    for pi in items {
        if let Some(slot) = location_to_slot(pi.item.location) {
            by_slot.entry(slot).or_insert(pi);
        }
    }
    EQUIPMENT_SLOTS.iter().map(|slot| {
        let pi = by_slot.get(slot).copied();
        let code = pi.map(|p| p.item.code.clone());
        let quality_opt = pi.map(|p| p.item.quality.as_u8());
        let (mut name_en, mut name_zh, name_zh_tw) = if let Some(pi) = pi {
            let en = resolver.resolve(conn, &pi.item.code, quality_opt, pi.item.unique_id, pi.item.set_id, "enUS");
            let zh = resolver.resolve(conn, &pi.item.code, quality_opt, pi.item.unique_id, pi.item.set_id, "zhCN");
            let tw = resolver.resolve(conn, &pi.item.code, quality_opt, pi.item.unique_id, pi.item.set_id, "zhTW");
            (Some(en.display_name), Some(zh.display_name), Some(tw.display_name))
        } else {
            (None, None, None)
        };
        // 符文之语名称追加 [隐秘]
        if let Some(p) = pi
            && p.item.flags.is_runeword() && !p.item.socketed_items.is_empty() {
                let rc: Vec<&str> = p.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
                if let Some(rw_en) = crate::data::runewords::match_runeword(&rc) {
                    name_en = Some(format!("{} [{}]", name_en.as_deref().unwrap_or("?"), rw_en));
                    if let Some(rw_zh) = crate::data::runewords::match_runeword_zh(&rc) {
                        name_zh = Some(format!("{}[{}]", name_zh.as_deref().unwrap_or("?"), rw_zh));
                    }
                }
            }
        let classified = if let Some(p) = pi {
            let stats_cat = crate::commands::character::categorize_item_stats(
                p.item.quality.as_u8(), p.item.flags.is_runeword(), &p.item.stat_lists);
            let mut td = crate::commands::character::build_tooltip_from_stats(
                &stats_cat, tooltip_language, Some(conn), resolver.profile_id);
            // 底材基础属性（防御、需求）
            // 实际防御优先: item body 解析出的 defense (已含 ED 加成);0 时回退底材区间
            let code = &p.item.code;
            if let Some(def) = crate::data::items_base::armor_stats(code) {
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
            if let Some((smin, smax)) = crate::data::items_base::shield_smite(code) {
                td.base_stats.push(if smin == smax {
                    format!("盾击伤害: {}", smin)
                } else {
                    format!("盾击伤害: {}-{}", smin, smax)
                });
            }
            if let Some(wpn) = crate::data::items_base::weapon_stats(code) {
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
            // 耐久度
            if p.item.max_durability > 0 {
                td.base_stats.push(format!("耐久度: {}/{}", p.item.current_durability, p.item.max_durability));
            }
            // 物品等级
            // 魔法词缀等级需求
            if p.item.quality.as_u8() == 4 && (p.magic_prefix_id.is_some() || p.magic_suffix_id.is_some()) {
                let req = crate::resource::queries::get_magic_item_req_level(
                    conn, resolver.profile_id, p.magic_prefix_id, p.magic_suffix_id);
                if req > 0 {
                    // 去掉之前可能已经加上的 base levelreq，用词的缀等级覆盖
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", req));
                }
            }
            // 暗金物品等级需求 (unique_item_def.level_req)
            if p.item.quality.as_u8() == 7
                && let Some(uid) = p.item.unique_id
                && let Some(def) = crate::resource::queries::get_unique_def(conn, resolver.profile_id, uid)
                && def.level_req > 0 {
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", def.level_req));
                }
            // 套装物品等级需求 (set_item_def.level_req)
            if p.item.quality.as_u8() == 5
                && let Some(sid) = p.item.set_id
                && let Some(def) = crate::resource::queries::get_set_item_by_item_id(conn, resolver.profile_id, sid)
                && def.level_req > 0 {
                    td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                    td.base_stats.push(format!("需要等级: {}", def.level_req));
                }
            // 套装加成 (绿色 set_bonus_stats, 来自 sets.txt)
            if p.item.quality.as_u8() == 5
                && let Some(sid) = p.item.set_id {
                    crate::commands::character::append_set_bonuses(
                        &mut td, conn, resolver.profile_id, sid, tooltip_language);
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
            // 孔位 + 镶嵌物
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
        } else {
            None
        };
        EquipmentSlotInfo {
            slot: (*slot).to_string(),
            occupied: pi.is_some(),
            code,
            name_zh, name_en, name_zh_tw,
            quality: quality_opt.and_then(crate::commands::character::quality_key_from_byte),
            socketed: pi.is_some_and(|p| p.item.flags.socketed()),
            skill_bonuses: pi.map_or(Vec::new(), |p| {
                let rp: Option<(&Connection, &NameResolver)> = Some((conn, resolver));
                let mut bonuses = Vec::new();
                for sl in &p.item.stat_lists {
                    bonuses.extend(crate::commands::character::extract_skill_bonuses_with_opt(&sl.stats, rp));
                }
                bonuses
            }),
            durability_cur: pi.map_or(0, |p| p.item.current_durability),
            durability_max: pi.map_or(0, |p| p.item.max_durability),
            stats: pi.map_or(crate::commands::character::ItemStats::default(), |p| {
                crate::commands::character::categorize_item_stats(
                    p.item.quality.as_u8(), p.item.flags.is_runeword(), &p.item.stat_lists)
            }),
            tooltip: classified,
        }
    }).collect()
}
