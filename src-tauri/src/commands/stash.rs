use crate::AppState;
use crate::commands::character::{SocketedItemInfo, SocketsInfo, TooltipData, classify_tooltip};
use crate::commands::config::resolve_excel_path;
use serde::{Deserialize, Serialize};
use tauri::State;
use rusqlite::Connection;

/// Response for stash read operations
#[derive(Debug, Serialize, Deserialize)]
pub struct StashReadResult {
    pub stash_name: String,
    pub stash_file: Option<String>,
    pub item_count: usize,
    pub read_status: String,
    pub items: Vec<StashItemResult>,
    pub pages: Vec<StashPageInfo>,
}

/// Information about a stash page (tab)
#[derive(Debug, Serialize, Deserialize)]
pub struct StashPageInfo {
    pub index: usize,
    pub is_stackable: bool,
    pub item_count: usize,
    pub label: String,
    /// Auto-detected grid width (max position_x + 1, min 10)
    pub grid_width: u8,
    /// Auto-detected grid height (max position_y + 1, min 10)
    pub grid_height: u8,
}

/// A socketed sub-item (gem/rune/jewel embedded in equipment).
#[derive(Debug, Serialize, Deserialize)]
pub struct StashSocketedItemInfo {
    pub code: String,
    pub item_name: String,
    pub quality: String,
    pub quantity: u32,
}

/// A single item from the stash, formatted for the frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct StashItemResult {
    /// Unique identifier within the stash read result.
    /// Format: `stash-{page_index}-{position_x}-{position_y}-{seq}` (32 chars).
    /// Stable across reads (assuming stash contents unchanged).
    pub id: String,
    pub item_name: String,
    /// 英文名称
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_en: Option<String>,
    pub quality: String,
    pub quantity: u32,
    pub code: String,
    pub kind: String,
    pub icon: String,
    pub position_x: u8,
    pub position_y: u8,
    pub inv_width: u8,
    pub inv_height: u8,
    pub page_index: usize,
    pub mod_added: bool,
    pub tooltip_lines: Vec<String>,
    /// 结构化 tooltip 数据（按 base_info / stats / hidden_info / set_info 分类）。
    /// 与装备/背包的 TooltipData 字段保持一致,前端 ItemTooltip 优先消费此字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<TooltipData>,
    /// Socketed sub-items (gems/runes/jewels embedded in this item).
    /// Empty for non-equipment or unsocketed items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub socketed_items: Vec<StashSocketedItemInfo>,
    /// 装备位 (D2 item 4b location_id):
    ///   0=None(容器存放), 1=Head, 2=Neck, 3=Torso, 4=RightHand, 5=LeftHand,
    ///   6=RightFinger, 7=LeftFinger, 8=Waist, 9=Feet, 10=Hands,
    ///   11=Trinket1(D2R), 12=Trinket2(D2R)。
    /// None = 解析失败;前端"部位"chip 按 u8 数字过滤。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_slot: Option<u8>,
    /// 底材类型(D2 txt type 列,如 "sword"/"axe"/"shield"/"helm"/"armor"/"spear"/...)。
    /// 来源:game_data_loader::get_item_def(code).item_type。
    /// None = game_data 未加载 / code 不在 vanilla 表 / 非装备类物品。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_type: Option<String>,
}

/// Find the shared stash file in a given save folder.
/// Tries modern names first (ModernSharedStashSoftCoreV2.d2i), then
/// legacy names (SharedStashSoftCoreV2.d2i) for older D2R installations.
/// Find the shared stash file in a given save folder.
/// If `mod_save_path` is set (non-empty), also checks the mod's subdirectory
/// (e.g. `save_folder/mods/仙道轮回/saves/`) for the stash file before falling
/// back to the save folder root — this mirrors D2R's mod save file routing.
fn find_shared_stash_file(save_folder: &str, mod_save_path: &str) -> Option<String> {
    let candidates = [
        "ModernSharedStashSoftCoreV2.d2i",
        "ModernSharedStashHardCoreV2.d2i",
        "SharedStashSoftCoreV2.d2i",
        "SharedStashHardCoreV2.d2i",
    ];

    // For mod stashes: D2R writes the stash file under save_folder/<mod_save_path>/.
    // E.g. save_folder/mods/仙道轮回/saves/ModernSharedStashSoftCoreV2.d2i
    if !mod_save_path.is_empty() {
        for filename in &candidates {
            let path = std::path::Path::new(save_folder)
                .join(mod_save_path)
                .join(filename);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    // Fallback: check save folder root (vanilla or unknown mod layout)
    for filename in &candidates {
        let path = std::path::Path::new(save_folder).join(filename);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}

#[tauri::command]
pub async fn read_stash(state: State<'_, AppState>) -> Result<StashReadResult, String> {
    let (save_folder, game_root, active_mod, language, profile_id, mod_save_path) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.get_config("save_folder").ok().flatten().unwrap_or_default(),
            db.get_config("game_root").ok().flatten().unwrap_or_default(),
            db.get_config("active_mod").ok().flatten().unwrap_or_default(),
            db.get_config("language").ok().flatten().unwrap_or_else(|| "enUS".to_string()),
            crate::commands::config::get_active_profile_id(&db).unwrap_or(0),
            db.get_config("mod_save_path").ok().flatten().unwrap_or_default(),
        )
    };

    let game_data_path = if !game_root.is_empty() {
        let path = resolve_excel_path(&game_root, &active_mod);
        println!("[D2R] resolved game_data_path='{}' language='{}'", path, language);
        if path.is_empty() { None } else { Some(path) }
    } else {
        println!("[D2R] no game_root configured, using embedded names");
        None
    };

    if save_folder.is_empty() {
        return Ok(StashReadResult {
            stash_name: "Shared Stash".into(), stash_file: None, item_count: 0,
            read_status: "Save folder not configured".into(), items: vec![], pages: vec![],
        });
    }

    let stash_path = find_shared_stash_file(&save_folder, &mod_save_path)
        .ok_or_else(|| "Shared stash file not found".to_string())?;

    // Keep the db lock alive during the entire operation to reuse the connection
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let stash_grid_size = db.get_config("stash_grid_size").ok().flatten()
        .and_then(|v| v.parse::<u8>().ok()).unwrap_or(10);
    let conn: &rusqlite::Connection = db.connection();

    read_stash_file_inner(&stash_path, game_data_path.as_deref(), &language, stash_grid_size, profile_id, conn)
}

/// Parse stash using the new two-pass parser (protocol::d2i::parser).
/// Returns ParsedItem groups (by page) + new Page metadata for grid layout.
fn parse_with_new_parser(stash_path: &str) -> Result<(Vec<crate::protocol::d2i::parser::ParsedItem>, Vec<crate::protocol::d2i::page::Page>), String> {
    let stash_file = crate::core::MmapFile::open(stash_path)
        .map_err(|e| format!("Failed to mmap stash file: {}", e))?;
    let data: &[u8] = stash_file.as_slice();
    let file = crate::protocol::d2i::parser::parse_file(data)
        .map_err(|e| format!("Failed to parse stash: {}", e))?;
    println!("[D2R] parse_with_new_parser: {} pages, {} items",
             file.pages.len(), file.items.len());
    for (i, p) in file.pages.iter().enumerate() {
        let count = file.items.iter().filter(|it| it.page_index == i).count();
        println!("[D2R]   page {}: is_stackable={} size={} items={}", i, p.is_stackable, p.size, count);
    }
    Ok((file.items, file.pages))
}

fn read_stash_file_inner(stash_path: &str, game_data_path: Option<&str>, language: &str, stash_grid_size: u8, profile_id: i64, conn: &Connection) -> Result<StashReadResult, String> {
    let _t0 = std::time::Instant::now();
    let mut ta = TimingAccum::new();
    println!("[D2R] read_stash_file_inner: path={}", stash_path);
    crate::data::stat_loader::set_runtime_excel_path(game_data_path);

    if let Some(path) = game_data_path
        && !path.is_empty() && !crate::protocol::d2i::legacy::game_data_loader::is_loaded() {
            let _t_init = std::time::Instant::now();
            crate::protocol::d2i::legacy::game_data_loader::initialize(path);
            eprintln!("[timing] game_data_loader::initialize {:?}", _t_init.elapsed());
        }

    let (parsed_items, raw_pages) = parse_with_new_parser(stash_path)?;
    ta.add("parse_items", _t0.elapsed());
    let total_before_filter = parsed_items.len();
    let socketed_hidden = parsed_items.iter().filter(|pi| pi.is_socketed_subitem).count();
    eprintln!("[D2R] read_stash: {} total ({} main, {} socketed)",
              total_before_filter, total_before_filter - socketed_hidden, socketed_hidden);

    let code_map = crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP;
    let inventory_sizes = crate::protocol::d2i::legacy::item_sizes::ITEM_INVENTORY_SIZES;

    let resolver_opt = {
        let _t_res = std::time::Instant::now();
        let resolver = crate::resource::get_cached_resolver(conn, profile_id);
        ta.add("name_resolver_init", _t_res.elapsed());
        Some((conn, resolver))
    };
    let mut page_infos: Vec<StashPageInfo> = Vec::new();
    let mut all_result_items: Vec<StashItemResult> = Vec::new();

    let mut page_max_indices: Vec<usize> = raw_pages.iter().map(|_| 0).collect();
    for pi in &parsed_items {
        if pi.page_index < page_max_indices.len() {
            page_max_indices[pi.page_index] += 1;
        }
    }

    for (page_index, page_opt) in raw_pages.iter().enumerate() {
        let page_items: Vec<&crate::protocol::d2i::parser::ParsedItem> = parsed_items.iter()
            .filter(|pi| pi.page_index == page_index).collect();
        if page_opt.is_stackable {
            log::debug!("[stash] 高级页 raw items before amount>0 filter:");
            for pi in &page_items {
                log::debug!("[stash]   code={} amount={} x={} y={}",
                    pi.item.code, pi.item.amount, pi.item.x, pi.item.y);
            }
        }
        let filtered_items: Vec<_> = page_items.iter()
            .filter(|pi| pi.item.amount > 0).collect();
        let item_count = filtered_items.len();

        let default_size = if page_opt.is_stackable { 10u8 } else { stash_grid_size };
        let mut max_x: u8 = default_size - 1;
        let mut max_y: u8 = default_size - 1;
        for pi in &filtered_items {
            if pi.item.x > max_x { max_x = pi.item.x; }
            if pi.item.y > max_y { max_y = pi.item.y; }
        }

        let label = if page_opt.is_stackable { "高级页".to_string() }
            else { format!("仓库 {}", page_index + 1) };

        log::debug!("[stash] page_index={} is_stackable={} item_count={} raw_pages_total={}",
            page_index, page_opt.is_stackable, item_count, raw_pages.len());

        if page_opt.is_stackable {
            log::debug!("[stash] 高级页(page_index={}) 物品清单: 共 {} 件", page_index, item_count);
            for pi in &filtered_items {
                let q = pi.item.quality.as_u8();
                let socketed_codes: Vec<&str> = pi.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
                log::debug!("[stash]   - code={} quality={} amount={} socketed={:?} page={:?} x={} y={}",
                    pi.item.code, q, pi.item.amount, socketed_codes, pi.item.page, pi.item.x, pi.item.y);
            }
        }

        let (grid_width, grid_height) = if page_opt.is_stackable {
            (default_size, default_size)
        } else {
            (max_x + 1, max_y + 1)
        };

        page_infos.push(StashPageInfo { index: page_index, is_stackable: page_opt.is_stackable, item_count, label, grid_width, grid_height });

        let mut seq_counter: u32 = 0;
        let auto_layout = page_opt.is_stackable;
        let mut layout_idx: u32 = 0;
        let _tp = std::time::Instant::now();
        for pi in &filtered_items {
            let item = &pi.item;
            let quality_u8 = Some(item.quality.as_u8());
            seq_counter += 1;
            let item_id = format!("stash-{}-{}-{}-{}", page_index, item.x, item.y, seq_counter);

            let socketed_items: Vec<StashSocketedItemInfo> = pi.item.socketed_items.iter().map(|si| {
                let sq = match si.quality.as_u8() { 7=>"unique",6=>"rare",5=>"set",4=>"magic",3=>"superior",1=>"low",_=>"normal" };
                StashSocketedItemInfo { code: si.code.clone(), item_name: si.code.clone(), quality: sq.to_string(), quantity: si.amount }
            }).collect();

            let (localized_name, en_name) = if let Some((conn, ref resolver)) = resolver_opt {
                let _tn = std::time::Instant::now();
                let resolved = resolver.resolve_with_affix(conn, &item.code, quality_u8, pi.item.unique_id, pi.item.set_id, None, None, &[], language);
                let en = resolver.resolve_with_affix(conn, &item.code, quality_u8, pi.item.unique_id, pi.item.set_id, None, None, &[], "enUS");
                ta.add("resolve_name", _tn.elapsed());
                (resolved.display_name, en.display_name)
            } else {
                (item.code.clone(), item.code.clone())
            };

            // Runeword suffix (matches character_equip.rs behavior)
            let (localized_name, en_name) = if pi.item.flags.is_runeword() && !pi.item.socketed_items.is_empty() {
                let rc: Vec<&str> = pi.item.socketed_items.iter().map(|si| si.code.as_str()).collect();
                if let Some(rw_en) = crate::data::runewords::match_runeword(&rc) {
                    (format!("{}[{}]", localized_name, crate::data::runewords::match_runeword_zh(&rc).unwrap_or(rw_en)),
                     format!("{} [{}]", en_name, rw_en))
                } else {
                    (localized_name, en_name)
                }
            } else {
                (localized_name, en_name)
            };

            let _tk = std::time::Instant::now();
            let kind = if crate::protocol::d2i::legacy::game_data_loader::is_grimoire_offhand(&item.code) { "shield".to_string() }
            else { match code_map.iter().find(|(c, _, _, _)| *c == item.code) {
                Some((_, _, k, _)) => k.to_string(),
                None => { crate::protocol::d2i::legacy::game_items::ALL_ITEMS.iter()
                    .find(|(c, _, _, _, _)| *c == item.code)
                    .map(|(_, _, is_a, is_w, is_s)| { if *is_a {"armor"} else if *is_w {"weapon"} else if *is_s {"shield"} else {"misc"} }.to_string())
                    .unwrap_or_else(|| "misc".to_string()) }
            }};
            let icon = match code_map.iter().find(|(c, _, _, _)| *c == item.code) {
                Some((_, _, _, i)) => i.to_string(),
                None => format!("/assets/img/items/default_{}.webp", kind),
            };
            let (inv_width, inv_height) = match inventory_sizes.iter().find(|(c, _, _)| *c == item.code) {
                Some((_, w, h)) => (*w, *h), None => (1, 1),
            };
            ta.add("lookup_meta", _tk.elapsed());
            let _ttool = std::time::Instant::now();
            let all_stats: Vec<crate::protocol::common::ItemStat> = item.stat_lists.iter()
                .flat_map(|sl| sl.stats.iter().cloned()).collect();
            let tooltip = crate::resource::TooltipFormatter::stash_tooltip(
                &localized_name, &en_name, &item.code, &kind, quality_u8, item.amount, language, &all_stats,
            );
            let categorized = crate::commands::character::categorize_item_stats(
                item.quality.as_u8(), pi.item.flags.is_runeword(), &item.stat_lists,
            );
            let conn_opt = resolver_opt.as_ref().map(|(conn, _)| *conn);
            let mut classified = if !tooltip.is_empty() {
                let fallback = classify_tooltip(&tooltip);
                let mut td = crate::commands::character::build_tooltip_from_stats(
                    &categorized, language, conn_opt, profile_id,
                );
                td.base_info = fallback.base_info;
                td.hidden_info = fallback.hidden_info;
                td.set_info = fallback.set_info;
                Some(td)
            } else { None };

            if let Some(td) = &mut classified {
                if item.quality.as_u8() == 4
                    && let Some((conn, resolver)) = &resolver_opt {
                        if let Some(pre) = pi.magic_prefix_id
                            && let Some(name) = resolver.get_affix_name(conn, pre, language) { td.affix_stats.push(name); }
                        if let Some(suf) = pi.magic_suffix_id
                            && let Some(name) = resolver.get_affix_name(conn, suf + 728, language) { td.affix_stats.push(name); }
                    }
                // Base stats (defense, requirements) — matches character_equip.rs
                // 实际防御优先: item body 解析出的 defense (已含 ED 加成);0 时回退底材区间
                if let Some(def) = crate::data::items_base::armor_stats(&item.code) {
                    if item.defense > 0 {
                        td.base_stats.insert(0, format!("防御: {}", item.defense));
                    } else if def.minac > 0 || def.maxac > 0 {
                        td.base_stats.insert(0, format!("防御: {}-{}", def.minac, def.maxac));
                    }
                    if def.levelreq > 0 { td.base_stats.push(format!("需要等级: {}", def.levelreq)); }
                    if def.reqstr > 0 { td.base_stats.push(format!("需要力量: {}", def.reqstr)); }
                    if def.reqdex > 0 { td.base_stats.push(format!("需要敏捷: {}", def.reqdex)); }
                }
                // 盾牌 smite 伤害 (含圣骑士盾)
                if let Some((smin, smax)) = crate::data::items_base::shield_smite(&item.code) {
                    td.base_stats.push(if smin == smax {
                        format!("盾击伤害: {}", smin)
                    } else {
                        format!("盾击伤害: {}-{}", smin, smax)
                    });
                }
                if let Some(wpn) = crate::data::items_base::weapon_stats(&item.code) {
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
                if item.max_durability > 0 {
                    td.base_stats.push(format!("耐久度: {}/{}", item.current_durability, item.max_durability));
                }
                // 暗金物品等级需求 (unique_item_def.level_req)
                if item.quality.as_u8() == 7
                    && let Some(uid) = item.unique_id
                    && let Some(conn) = conn_opt
                    && let Some(def) = crate::resource::queries::get_unique_def(conn, profile_id, uid)
                    && def.level_req > 0 {
                        td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                        td.base_stats.push(format!("需要等级: {}", def.level_req));
                    }
                // 套装物品等级需求 (set_item_def.level_req)
                if item.quality.as_u8() == 5
                    && let Some(sid) = item.set_id
                    && let Some(conn) = conn_opt
                    && let Some(def) = crate::resource::queries::get_set_item_by_item_id(conn, profile_id, sid)
                    && def.level_req > 0 {
                        td.base_stats.retain(|l| !l.starts_with("需要等级:"));
                        td.base_stats.push(format!("需要等级: {}", def.level_req));
                    }
                // 套装加成 (绿色 set_bonus_stats, 来自 sets.txt)
                if item.quality.as_u8() == 5
                    && let Some(sid) = item.set_id
                    && let Some(conn) = conn_opt {
                        crate::commands::character::append_set_bonuses(
                            td, conn, profile_id, sid, language);
                    }
            }
            ta.add("tooltip_build", _ttool.elapsed());

            if item.num_sockets > 0 {
                let socketed_infos: Vec<SocketedItemInfo> = item.socketed_items.iter().map(|si| {
                    let q = si.quality.as_u8();
                    SocketedItemInfo { code: si.code.clone(), name_zh: None, name_en: None,
                        quality: Some(q).filter(|&v| (2..=8).contains(&v)), amount: si.amount, }
                }).collect();
                if let Some(ref mut c) = classified {
                    c.sockets = Some(SocketsInfo { count: item.num_sockets, items: socketed_infos });
                }
            }

            let mod_added = crate::protocol::d2i::legacy::game_data_loader::is_mod_item(&item.code);
            let equipment_slot = Some(item.location.as_u8());
            let base_type = crate::protocol::d2i::legacy::game_data_loader::get_item_def(&item.code)
                .map(|def| def.item_type.clone()).filter(|s| !s.is_empty());

            let (pos_x, pos_y) = if auto_layout {
                let x = (layout_idx % default_size as u32) as u8;
                let y = (layout_idx / default_size as u32) as u8;
                layout_idx += 1;
                (x, y)
            } else { (item.x, item.y) };

            all_result_items.push(StashItemResult {
                id: item_id, item_name: localized_name, name_en: Some(en_name),
                quality: quality_u8.map(|q| match q {7=>"unique",6=>"rare",5=>"set",4=>"magic",_=>"normal"}).unwrap_or("normal").into(),
                quantity: item.amount, code: item.code.clone(), kind, icon,
                position_x: pos_x, position_y: pos_y, inv_width, inv_height,
                page_index, mod_added, tooltip_lines: tooltip, tooltip: classified,
                socketed_items, equipment_slot, base_type,
            });
        }
        ta.add("build_items", _tp.elapsed());
        eprintln!("[timing] stash page={} build_items {:?} ({} filtered)",
            page_index, _tp.elapsed(), filtered_items.len());

        if page_index == 0 {
            for (idx, pi) in filtered_items.iter().take(5).enumerate() {
                log::debug!("[stash] page[0] grid {}x{} [{}] code={} x={} y={}",
                    grid_width, grid_height, idx, pi.item.code, pi.item.x, pi.item.y);
            }
        }
    }

    // ★ Measure JSON serialization time
    let _tser = std::time::Instant::now();
    let result = StashReadResult {
        stash_name: "Shared Stash".into(),
        stash_file: Some(stash_path.to_string()),
        item_count: all_result_items.len(),
        read_status: "Stash successfully read".into(),
        items: all_result_items,
        pages: page_infos,
    };
    ta.add("serialize", _tser.elapsed());
    ta.add("total", _t0.elapsed());
    ta.dump();
    Ok(result)
}

/// Simple per-category timing accumulator for stash read profiling.
struct TimingAccum {
    entries: std::collections::HashMap<&'static str, (usize, std::time::Duration)>,
}
impl TimingAccum {
    fn new() -> Self { Self { entries: std::collections::HashMap::new() } }
    fn add(&mut self, label: &'static str, elapsed: std::time::Duration) {
        let (count, sum) = self.entries.entry(label).or_insert((0, std::time::Duration::ZERO));
        *count += 1;
        *sum += elapsed;
    }
    fn dump(&self) {
        eprintln!();
        eprintln!("[timing] ═══════════════════ Stash Timing Breakdown ═══════════════════");
        let mut pairs: Vec<_> = self.entries.iter().map(|(k, v)| (*k, v.0, v.1)).collect();
        pairs.sort_by_key(|(_, _, d)| std::cmp::Reverse(*d));
        for (label, count, total) in &pairs {
            let avg = if *count > 0 { *total / *count as u32 } else { std::time::Duration::ZERO };
            eprintln!("[timing]   {:30} {:>12?}  ({} calls, avg {:?})", label, total, count, avg);
        }
        eprintln!("[timing] ══════════════════════════════════════════════════════════════");
    }
}

/// Create a backup of the stash files
#[tauri::command]
pub fn create_stash_backup(state: State<AppState>) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db
        .get_config("save_folder")
        .ok()
        .flatten()
        .unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Ok(BackupResult {
            success: true,
            backup_count: 0,
            message: "Save folder not configured.".into(),
        });
    }

    let save_path = std::path::Path::new(&save_folder);

    // Scan for save/backup related files
    let found: Vec<String> = std::fs::read_dir(save_path)
        .map_err(|e| format!("Failed to read save folder: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| {
            e.path().extension()
                .map(|ext| {
                    ext == "d2i" || ext == "d2s" || ext == "json"
                    || ext == "fltr" || ext == "ctl" || ext == "key"
                    || ext == "ma0" || ext == "map"
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    if found.is_empty() {
        return Ok(BackupResult {
            success: true,
            backup_count: 0,
            message: "No save files found to back up.".into(),
        });
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_dir = save_path
        .join("marketplace_backups")
        .join(timestamp.to_string());

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let mut count = 0;
    for src in &found {
        let filename = std::path::Path::new(src)
            .file_name()
            .unwrap()
            .to_string_lossy();
        let dest = backup_dir.join(filename.as_ref());
        std::fs::copy(src, &dest)
            .map_err(|e| format!("Failed to copy {}: {}", src, e))?;
        count += 1;
    }

    Ok(BackupResult {
        success: true,
        backup_count: count,
        message: format!("Backup created with {} file(s)", count),
    })
}


/// Result of extracting character equipment into the warehouse.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResult {
    pub extracted_count: usize,
    pub warehouse_ids: Vec<String>,
    pub page_name: String,
    pub source_character: String,
    pub equipped_count: usize,
    pub backpack_count: usize,
    pub belt_count: usize,
    pub skipped_items: Vec<SkippedItemReason>,
}

/// Reason an item was skipped during extraction.
#[derive(Debug, Serialize, Deserialize)]
pub struct SkippedItemReason {
    pub item_name: String,
    pub reason: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub backup_count: usize,
    pub message: String,
}

/// Per-file info in a backup entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupFileInfo {
    pub filename: String,
    pub file_type: String,    // "character" | "stash" | "config" | "other"
    pub backup_size: u64,
    pub current_size: Option<u64>,
}

fn classify_file_type(filename: &str) -> String {
    if filename.ends_with(".d2s") { "character".into() }
    else if filename.ends_with(".d2i") || filename.ends_with(".d2x") { "stash".into() }
    else if filename.ends_with(".json") { "config".into() }
    else { "other".into() }
}

/// A single backup entry
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupEntry {
    pub timestamp: String,
    pub path: String,
    pub files: Vec<BackupFileInfo>,
    pub total_size: u64,
    pub created_at: String,
}

/// List all available backups — returns enriched file info (size, type, current size).
#[tauri::command]
pub fn list_backups(state: State<AppState>) -> Result<Vec<BackupEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Ok(vec![]);
    }

    let save_path = std::path::Path::new(&save_folder);
    let backup_root = save_path.join("marketplace_backups");
    if !backup_root.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<BackupEntry> = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(&backup_root)
        .map_err(|e| format!("Failed to read backups: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| {
            let name = e.file_name();
            name != "auto"
            && name != "auto_save"
            && !name.to_string_lossy().starts_with("_pre_restore")
        })
        .collect();

    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for entry in dirs {
        let dirname = entry.file_name().to_string_lossy().to_string();
        let dirpath = entry.path();

        let mut files: Vec<BackupFileInfo> = Vec::new();
        let mut total_size: u64 = 0;

        if let Ok(rd) = std::fs::read_dir(&dirpath) {
            for fe in rd.filter_map(|e| e.ok()) {
                if !fe.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }
                let filename = fe.file_name().to_string_lossy().to_string();
                let file_type = classify_file_type(&filename);
                let backup_size = fe.metadata().ok().map(|m| m.len()).unwrap_or(0);
                total_size += backup_size;

                // Check current file in save folder
                let current_path = save_path.join(&filename);
                let current_size = if current_path.exists() {
                    std::fs::metadata(&current_path).ok().map(|m| m.len())
                } else {
                    None
                };

                files.push(BackupFileInfo { filename, file_type, backup_size, current_size });
            }
        }

        entries.push(BackupEntry {
            timestamp: dirname.clone(),
            path: dirpath.to_string_lossy().to_string(),
            files,
            total_size,
            created_at: dirname.clone(),
        });
    }


    Ok(entries)
}
/// Restore backup files — optionally restore only specified files.
/// If `files` is None or empty, restores all files in the backup.
#[tauri::command]
pub fn restore_backup(
    state: State<AppState>,
    timestamp: String,
    files: Option<Vec<String>>,
) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    let save_path = std::path::Path::new(&save_folder);
    let backup_dir = save_path
        .join("marketplace_backups")
        .join(&timestamp);

    if !backup_dir.exists() {
        return Err(format!("Backup '{}' not found", timestamp));
    }

    // Read files from backup dir, optionally filtered
    let all_files: Vec<String> = std::fs::read_dir(&backup_dir)
        .map_err(|e| format!("Failed to read backup dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let restore_targets: Vec<&str> = match &files {
        Some(list) if !list.is_empty() => list.iter().map(|s| s.as_str()).collect(),
        _ => all_files.iter().map(|s| s.as_str()).collect(),
    };

    if restore_targets.is_empty() {
        return Err("No files to restore".into());
    }

    // Create a safety backup of current files that will be overwritten
    let safety_ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let safety_dir = save_path
        .join("marketplace_backups")
        .join(format!("_pre_restore_{}", safety_ts));
    std::fs::create_dir_all(&safety_dir)
        .map_err(|e| format!("Failed to create safety backup: {}", e))?;

    for filename in &restore_targets {
        let src = save_path.join(filename);
        if src.exists() {
            let dest = safety_dir.join(filename);
            let _ = std::fs::copy(&src, &dest);
        }
    }

    // Restore selected files
    let mut count = 0usize;
    for filename in &restore_targets {
        if !all_files.contains(&filename.to_string()) {
            return Err(format!("File '{}' not found in backup", filename));
        }
        let src = backup_dir.join(filename);
        let dest = save_path.join(filename);
        std::fs::copy(&src, &dest)
            .map_err(|e| format!("Failed to restore {}: {}", filename, e))?;
        count += 1;
    }

    Ok(BackupResult {
        success: true,
        backup_count: count,
        message: format!("Restored {} file(s) from backup. A safety copy was saved.", count),
    })
}
/// An auto-backup entry (created before deposit/withdraw).
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoBackupEntry {
    pub filename: String,
    pub original_stash: String,
    pub operation: String,
    pub timestamp: String,
    pub size: u64,
    pub path: String,
}

/// List auto-backups from marketplace_backups/auto/.
#[tauri::command]
pub fn list_auto_backups(state: State<AppState>) -> Result<Vec<AutoBackupEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Ok(vec![]);
    }

    let auto_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups").join("auto");
    if !auto_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<AutoBackupEntry> = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&auto_dir)
        .map_err(|e| format!("Failed to read auto backups: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    // Sort by modified time (newest first)
    files.sort_by(|a, b| {
        b.metadata().and_then(|m| m.modified()).ok()
            .cmp(&a.metadata().and_then(|m| m.modified()).ok())
    });

    for path in &files {
        let filename = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Parse filename: "<original>.<operation>_<timestamp>.bak"
        let (original_stash, operation, ts) = parse_auto_backup_name(&filename);

        let size = path.metadata().ok().map(|m| m.len()).unwrap_or(0);

        entries.push(AutoBackupEntry {
            filename,
            original_stash,
            operation,
            timestamp: ts,
            size,
            path: path.to_string_lossy().to_string(),
        });
    }

    Ok(entries)
}

/// Parse an auto-backup filename into (original_stash, operation, timestamp).
fn parse_auto_backup_name(filename: &str) -> (String, String, String) {
    // Strip .bak suffix
    let name = filename.strip_suffix(".bak").unwrap_or(filename);
    // Split on ".deposit_", ".withdraw_", or ".restore_"
    for op in &["deposit", "withdraw", "restore"] {
        let marker = format!(".{}_", op);
        if let Some(pos) = name.rfind(&marker) {
            let original = name[..pos].to_string();
            let ts = name[pos + marker.len()..].to_string();
            return (original, op.to_string(), ts);
        }
    }
    (name.to_string(), "unknown".into(), String::new())
}

/// Restore a single file from an auto-backup.
/// The backup file is copied back to the save folder under its original name.
#[tauri::command]
pub fn restore_auto_backup(
    state: State<AppState>,
    backup_filename: String,
) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }


    let auto_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups").join("auto");
    let src = auto_dir.join(&backup_filename);

    if !src.exists() {
        return Err(format!("Auto-backup '{}' not found", backup_filename));
    }

    // Parse original filename from the backup name
    let (original_stash, _, _) = parse_auto_backup_name(&backup_filename);
    let dest = std::path::Path::new(&save_folder).join(&original_stash);

    if !dest.exists() {
        return Err(format!("Original stash file '{}' not found in save folder", original_stash));
    }

    // Safety backup before overwriting
    let safety_ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let safety_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups")
        .join(format!("_pre_restore_{}", safety_ts));
    std::fs::create_dir_all(&safety_dir)
        .map_err(|e| format!("Failed to create safety backup: {}", e))?;
    let safety_dest = safety_dir.join(&original_stash);
    std::fs::copy(&dest, &safety_dest)
        .map_err(|e| format!("Failed to create safety copy: {}", e))?;

    // Restore
    std::fs::copy(&src, &dest)
        .map_err(|e| format!("Failed to restore '{}': {}", original_stash, e))?;

    Ok(BackupResult {
        success: true,
        backup_count: 1,
        message: format!("Restored {} from auto-backup. Safety copy saved.", original_stash),
    })
}

/// Clean up auto-backups older than keep_days.
#[tauri::command]
pub fn cleanup_auto_backups(
    state: State<AppState>,
    keep_days: Option<u32>,
) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    let auto_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups").join("auto");
    if !auto_dir.exists() {
        return Ok(BackupResult {
            success: true,
            backup_count: 0,
            message: "No auto-backups to clean.".into(),
        });
    }

    let keep = keep_days.unwrap_or(30);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(keep as i64);
    let mut removed = 0usize;

    if let Ok(rd) = std::fs::read_dir(&auto_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
                    && let Ok(modified) = meta.modified() {
                        let modified_system: chrono::DateTime<chrono::Utc> =
                            chrono::DateTime::from(modified);
                        if modified_system < cutoff {
                            let path = entry.path();
                            if std::fs::remove_file(&path).is_ok() {
                                removed += 1;
                            }
                        }
                    }
        }
    }

    Ok(BackupResult {
        success: true,
        backup_count: removed,
        message: format!("Cleaned up {} old auto-backup(s).", removed),
    })
}

/// Delete a manual backup directory by timestamp.
#[tauri::command]
pub fn delete_backup(state: State<AppState>, timestamp: String) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    let backup_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups")
        .join(&timestamp);

    if !backup_dir.exists() {
        return Err(format!("Backup '{}' not found", timestamp));
    }

    // Count files before deleting
    let count = std::fs::read_dir(&backup_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    std::fs::remove_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to delete backup '{}': {}", timestamp, e))?;

    Ok(BackupResult {
        success: true,
        backup_count: count,
        message: format!("Deleted backup '{}' ({} file(s)).", timestamp, count),
    })
}
/// Auto-save stash files (overwrite mode).
/// Copies .d2i / .d2s files to marketplace_backups/auto_save/,
/// keeping only the latest copy. Designed to be called periodically.
#[tauri::command]
pub fn auto_save_stash(state: State<AppState>) -> Result<BackupResult, String> {
    use std::time::Instant;
    let _t0 = Instant::now();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    log::info!("[auto_save] ENTRY save_folder='{}' ({:.2?})", save_folder, _t0.elapsed());

    if save_folder.is_empty() {
        log::warn!("[auto_save] save_folder empty, skipping");
        return Ok(BackupResult { success: true, backup_count: 0, message: "Save folder not configured.".into() });
    }

    let save_path = std::path::Path::new(&save_folder);
    log::info!("[auto_save] save_path exists={} is_dir={} ({:.2?})",
        save_path.exists(), save_path.is_dir(), _t0.elapsed());

    // List top-level entries for debugging
    if let Ok(rd) = std::fs::read_dir(save_path) {
        let entries: Vec<String> = rd.filter_map(|e| e.ok()).filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string()).collect();
        log::info!("[auto_save] save_path contains {} file(s): {:?} ({:.2?})", entries.len(), entries, _t0.elapsed());
    } else {
        log::error!("[auto_save] FAILED to read save_path directory ({:.2?})", _t0.elapsed());
    }

    let auto_dir = save_path.join("marketplace_backups").join("auto_save");
    std::fs::create_dir_all(&auto_dir).ok();
    log::info!("[auto_save] auto_dir='{}' ({:.2?})", auto_dir.display(), _t0.elapsed());

    let latest_dir = auto_dir.join("latest");
    std::fs::create_dir_all(&latest_dir).ok();

    let patterns = ["d2i", "d2s"];
    let mut file_count = 0u32;
    if let Ok(rd) = std::fs::read_dir(&save_folder) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !patterns.contains(&ext) { continue; }
            let filename = entry.file_name().to_string_lossy().to_string();
            let dest = latest_dir.join(&filename);
            match std::fs::copy(&path, &dest) {
                Ok(n) => { file_count += 1; log::info!("[auto_save] copied {} -> {} ({}B)", filename, dest.display(), n); }
                Err(e) => { log::warn!("[auto_save] FAILED to copy {}: {}", filename, e); }
            }
        }
    }
    log::info!("[auto_save] copied {} file(s) to latest/ ({:.2?})", file_count, _t0.elapsed());

    let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    // Check for recent zip
    let mut need_new_zip = true;
    if let Ok(rd) = std::fs::read_dir(&auto_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".zip") { continue; }
            if let Ok(meta) = entry.metadata()
                && let Ok(modified) = meta.modified()
                    && let Ok(elapsed) = modified.elapsed()
                        && elapsed.as_secs() < 600 {
                            need_new_zip = false;
                        }
        }
    }

    if need_new_zip && file_count > 0 {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let zip_path = auto_dir.join(format!("{}.zip", ts));
        log::info!("[auto_save] creating zip {} ({:.2?})", zip_path.display(), _t0.elapsed());
        if let Ok(file) = std::fs::File::create(&zip_path) {
            use std::io::Write;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);
            if let Ok(rd) = std::fs::read_dir(&latest_dir) {
                for entry in rd.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_file() { continue; }
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if let Ok(mut src) = std::fs::File::open(&path)
                        && zip.start_file(&fname, options).is_ok() {
                            let mut buf = Vec::new();
                            let _ = std::io::Read::read_to_end(&mut src, &mut buf);
                            let _ = zip.write_all(&buf);
                        }
                }
            }
            let _ = zip.finish();
        }
    }

    // Cleanup old zips
    let cutoff = now_epoch.saturating_sub(1800);
    let mut cleaned = 0u32;
    if let Ok(rd) = std::fs::read_dir(&auto_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".zip") { continue; }
            let ts_str = &name[..name.len() - 4];
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y%m%d_%H%M%S") {
                let zip_epoch = parsed.and_utc().timestamp() as u64;
                if zip_epoch < cutoff {
                    let _ = std::fs::remove_file(entry.path());
                    cleaned += 1;
                }
            }
        }
    }

    let msg = format!("latest: {} file(s), cleaned {} old zip(s).", file_count, cleaned);
    log::info!("[auto_save] DONE: {} ({:.2?})", msg, _t0.elapsed());
    Ok(BackupResult { success: true, backup_count: file_count as usize, message: msg })
}
/// Get the auto-backup retention days setting (default: 5).
#[tauri::command]
pub fn get_auto_backup_retention(state: State<AppState>) -> Result<u32, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let val = db.get_config("auto_backup_retention_days")
        .ok().flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(5);
    Ok(val)
}

/// Set the auto-backup retention days.
#[tauri::command]
pub fn set_auto_backup_retention(state: State<AppState>, days: u32) -> Result<(), String> {
    let days = days.max(1).min(365);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("auto_backup_retention_days", &days.to_string())
        .map_err(|e| e.to_string())
}

/// Archive old auto-backups: files older than retention_days are grouped
/// by month into .zip archives, then the individual files are removed.
#[tauri::command]
pub fn archive_old_auto_backups(
    state: State<AppState>,
    retention_days: Option<u32>,
) -> Result<BackupResult, String> {
    use std::io::{Read, Write};

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    let configured_days = db.get_config("auto_backup_retention_days")
        .ok().flatten()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    let keep_days = retention_days.unwrap_or(configured_days as u32) as u64;
    let cutoff = chrono::Utc::now() - chrono::Duration::days(keep_days as i64);

    let auto_dir = std::path::Path::new(&save_folder)
        .join("marketplace_backups").join("auto");
    if !auto_dir.exists() {
        return Ok(BackupResult {
            success: true, backup_count: 0,
            message: "No auto backups to archive.".into(),
        });
    }

    // Collect files older than cutoff, grouped by month
    use std::collections::BTreeMap;
    let mut monthly: BTreeMap<String, Vec<std::path::PathBuf>> = BTreeMap::new();

    if let Ok(rd) = std::fs::read_dir(&auto_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            // Skip existing archives
            if path.extension().and_then(|e| e.to_str()) == Some("zip") { continue; }
            if let Ok(meta) = path.metadata()
                && let Ok(modified) = meta.modified() {
                    let dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::from(modified);
                    if dt < cutoff {
                        let key = dt.format("%Y-%m").to_string();
                        monthly.entry(key).or_default().push(path);
                    }
                }
        }
    }

    if monthly.is_empty() {
        return Ok(BackupResult {
            success: true, backup_count: 0,
            message: "No files exceed the retention period.".into(),
        });
    }

    let mut total_archived = 0usize;
    let mut total_removed = 0usize;

    for (month, files) in &monthly {
        let zip_path = auto_dir.join(format!("auto_{}.zip", month));
        // Skip if zip already exists for this month
        if zip_path.exists() { continue; }

        let file = std::fs::File::create(&zip_path)
            .map_err(|e| format!("Failed to create zip {}: {}", zip_path.display(), e))?;
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for path in files {
            let filename = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let mut src = std::fs::File::open(path)
                .map_err(|e| format!("Failed to open {}: {}", filename, e))?;
            zip_writer.start_file(&filename, options)
                .map_err(|e| format!("Zip write error for {}: {}", filename, e))?;
            let mut buf = Vec::new();
            src.read_to_end(&mut buf)
                .map_err(|e| format!("Read error for {}: {}", filename, e))?;
            zip_writer.write_all(&buf)
                .map_err(|e| format!("Zip write error for {}: {}", filename, e))?;
            total_archived += 1;
        }

        zip_writer.finish()
            .map_err(|e| format!("Failed to finalize zip {}: {}", zip_path.display(), e))?;

        // Delete individual files after successful archiving
        for path in files {
            if std::fs::remove_file(path).is_ok() {
                total_removed += 1;
            }
        }
    }

    let msg = if total_archived > 0 {
        format!("Archived {} file(s) into {} monthly zip(s), cleaned up {} file(s).",
            total_archived, monthly.len(), total_removed)
    } else {
        "All old files already archived.".into()
    };

    Ok(BackupResult {
        success: true,
        backup_count: total_archived,
        message: msg,
    })
}
/// Get info about the current auto_save snapshot (10s timer overwrite).
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoSaveInfo {
    pub file_count: usize,
    pub total_size: u64,
    pub files: Vec<String>,
    pub timestamp: String,
}
#[tauri::command]
pub fn get_auto_save_info(state: State<AppState>) -> Result<AutoSaveInfo, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Ok(AutoSaveInfo { file_count: 0, total_size: 0, files: vec![], timestamp: String::new() });
    }

    let latest_dir = std::path::Path::new(&save_folder).join("marketplace_backups").join("auto_save").join("latest");
    if !latest_dir.exists() {
        return Ok(AutoSaveInfo { file_count: 0, total_size: 0, files: vec![], timestamp: String::new() });
    }

    let mut total_size = 0u64;
    let mut files = Vec::new();
    let mut latest_mtime: Option<std::time::SystemTime> = None;

    if let Ok(rd) = std::fs::read_dir(&latest_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            if !entry.path().is_file() { continue; }
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
                if let Ok(mtime) = meta.modified()
                    && latest_mtime.is_none_or(|t| mtime > t) {
                        latest_mtime = Some(mtime);
                    }
            }
            files.push(filename);
        }
    }

    let timestamp = latest_mtime
        .map(|t| { let dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from(t); dt.format("%Y%m%d_%H%M%S").to_string() })
        .unwrap_or_default();

    Ok(AutoSaveInfo { file_count: files.len(), total_size, files, timestamp })
}

/// Restore all files from the auto_save snapshot back to the save folder.
#[tauri::command]
pub fn restore_auto_save(state: State<AppState>) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    let latest_dir = std::path::Path::new(&save_folder).join("marketplace_backups").join("auto_save").join("latest");
    if !latest_dir.exists() {
        return Err("No auto_save snapshot found".into());
    }

    let safety_ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let safety_dir = std::path::Path::new(&save_folder).join("marketplace_backups").join(format!("_pre_restore_{}", safety_ts));
    std::fs::create_dir_all(&safety_dir).ok();

    let mut count = 0u32;
    if let Ok(rd) = std::fs::read_dir(&latest_dir) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            let filename = entry.file_name().to_string_lossy().to_string();
            let current = std::path::Path::new(&save_folder).join(&filename);
            if current.exists() {
                let _ = std::fs::copy(&current, safety_dir.join(&filename));
            }
            if std::fs::copy(&path, &current).is_ok() {
                count += 1;
            }
        }
    }

    Ok(BackupResult { success: true, backup_count: count as usize, message: format!("Restored {} file(s) from auto_save snapshot.", count) })
}
/// A safety backup entry (created before restore operations).
#[derive(Debug, Serialize, Deserialize)]
pub struct SafetyBackupEntry {
    pub dirname: String,
    pub files: Vec<String>,
    pub file_count: usize,
    pub total_size: u64,
    pub timestamp: String,
}

/// List safety backups (_pre_restore_* directories).
#[tauri::command]
pub fn list_safety_backups(state: State<AppState>) -> Result<Vec<SafetyBackupEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
    drop(db);

    if save_folder.is_empty() {
        return Ok(vec![]);
    }

    let backup_root = std::path::Path::new(&save_folder).join("marketplace_backups");
    if !backup_root.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<SafetyBackupEntry> = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(&backup_root)
        .map_err(|e| format!("Failed to read backups: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.file_name().to_string_lossy().starts_with("_pre_restore"))
        .collect();

    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for entry in dirs {
        let dirname = entry.file_name().to_string_lossy().to_string();
        let dirpath = entry.path();
        let mut files = Vec::new();
        let mut total_size = 0u64;

        if let Ok(rd) = std::fs::read_dir(&dirpath) {
            for fe in rd.filter_map(|e| e.ok()) {
                if !fe.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }
                let filename = fe.file_name().to_string_lossy().to_string();
                total_size += fe.metadata().ok().map(|m| m.len()).unwrap_or(0);
                files.push(filename);
            }
        }

        let fcount = files.len();
        let ts = dirname.strip_prefix("_pre_restore_").unwrap_or(&dirname).to_string();
        entries.push(SafetyBackupEntry {
            dirname,
            files,
            file_count: fcount,
            total_size,
            timestamp: ts,
        });
    }

    Ok(entries)
}
