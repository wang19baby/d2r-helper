use crate::database::VirtualItem;
use crate::market::{
    calculate_sell_after_seconds, calculate_sell_price, get_market_reference_price,
    get_sell_price_suggestion, looks_like_rune, normalize_item_type,
};
use crate::protocol::d2i::legacy::constants;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone)]
struct MarketProfileContext {
    profile_id: i64,
    profile_key: String,
    game_version: String,
    active_mod: String,
}

/// Resolve item code from item name using the ITEM_NAME_TO_CODE table
fn resolve_item_code(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    constants::ITEM_NAME_TO_CODE.iter()
        .find(|(n, _)| *n == lower)
        .map(|(_, code)| *code)
}

fn get_market_profile_context(db: &crate::database::Database) -> MarketProfileContext {
    MarketProfileContext {
        profile_id: crate::commands::config::get_active_profile_id(db).unwrap_or(0),
        profile_key: crate::commands::config::get_active_profile_key(db).unwrap_or_default(),
        game_version: db.get_config("game_version").ok().flatten().unwrap_or_default(),
        active_mod: db.get_config("active_mod").ok().flatten().unwrap_or_default(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceSuggestionResult {
    pub base_price: i64,
    pub suggested_price: i64,
    pub min_price: i64,
    pub max_price: i64,
    pub variation: i64,
    pub has_reference: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyResult {
    pub new_balance: i64,
    pub item_id: String,
    pub stash_path: String,
}


/// Sell an item from the marketplace back to tokens
#[tauri::command]
pub fn sell_item(state: State<AppState>, item_id: String) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);

    let item = db
        .get_virtual_item_by_id(&item_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Item not found".to_string())?;

    let token_price = item.token_price.unwrap_or(0);
    let sell_price = calculate_sell_price(token_price);

    db.mark_item_as_sold(&item_id, &ctx.profile_key).map_err(|e| e.to_string())?;
    db.update_token_balance(sell_price)
        .map_err(|e| e.to_string())?;
    db.add_transaction(
        "sell",
        Some(&item_id),
        sell_price,
        &format!("Sold for {} tokens", sell_price),
    )
    .map_err(|e| e.to_string())?;

    db.get_token_balance().map_err(|e| e.to_string())
}

/// Buy an item from the catalog (purchases go directly to stash)
#[tauri::command]
pub fn buy_item(
    state: State<AppState>,
    item_name: String,
    item_kind: String,
    token_price: i64,
    qty: i32,
) -> Result<BuyResult, String> {
    if token_price <= 0 {
        return Err("token_price must be greater than zero".into());
    }
    let qty = if qty <= 0 { 1 } else { qty };
    let total_price = token_price * qty as i64;

    let normalized_item_kind = normalize_item_type(&item_kind);
    if !looks_like_rune(&item_name, &normalized_item_kind)
        && !crate::market::trade_rules::is_purchasable_type(Some(&normalized_item_kind))
    {
        return Err(format!(
            "Direct purchase not supported for this item type. Received: {} ({})",
            item_name, item_kind
        ));
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);

    // Check balance
    let balance = db.get_token_balance().map_err(|e| e.to_string())?;
    if balance < total_price {
        return Err("Insufficient balance".into());
    }

    // Resolve item code
    let item_code = resolve_item_code(&item_name)
        .ok_or_else(|| format!("Unknown item: {}", item_name))?;

    // Read the save folder from config
    let save_folder = db
        .get_config("save_folder")
        .ok()
        .flatten()
        .unwrap_or_default();

    if save_folder.is_empty() {
        return Err("Save folder not configured".into());
    }

    // Find stash file and modify items
    let stash_path = crate::services::StashService::resolve_stash_path(&save_folder)
        .ok_or_else(|| "Shared stash not found".to_string())?;
    crate::services::StashService::modify_stackable(&stash_path, item_code, qty)?;

    // Record the purchase
    let item_id = uuid::Uuid::new_v4().to_string();
    let purchase_item = VirtualItem {
        id: item_id.clone(),
        name: item_name.clone(),
        item_code: Some(item_code.to_string()),
        item_kind: Some(normalized_item_kind.clone()),
        item_type: Some(normalized_item_kind.clone()),
        quality: Some("normal".into()),
        level: None,
        attributes: Some(
            serde_json::json!({
                "name": item_name,
                "type": normalized_item_kind,
                "quantity": qty,
                "delivered_to_stash": true,
            })
            .to_string(),
        ),
        source: Some("purchased".into()),
        exported_from: None,
        purchased_at: Some(chrono::Utc::now().to_rfc3339()),
        token_price: Some(total_price),
        status: Some("imported".into()),
        quantity: Some(qty as i64),
        unit_price: Some(token_price),
        listed_at: None,
        sell_after_seconds: None,
        profile_id: Some(ctx.profile_id),
        profile_key: Some(ctx.profile_key.clone()),
        game_version: Some(ctx.game_version),
        mod_name: Some(ctx.active_mod),
    };

    db.add_virtual_item(&purchase_item)
        .map_err(|e| e.to_string())?;
    db.mark_item_as_imported(&item_id, &ctx.profile_key)
        .map_err(|e| e.to_string())?;
    db.update_token_balance(-total_price)
        .map_err(|e| e.to_string())?;
    db.add_transaction(
        "buy_import",
        Some(&item_id),
        -total_price,
        &format!("Purchased and delivered: {}x {}", qty, item_name),
    )
    .map_err(|e| e.to_string())?;

    let new_balance = db.get_token_balance().map_err(|e| e.to_string())?;

    Ok(BuyResult {
        new_balance,
        item_id,
        stash_path,
    })
}

/// Cancel a listing (restore items to stash)
#[tauri::command]
pub fn cancel_listing(state: State<AppState>, listing_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);

    let item = db
        .get_listed_item_by_id_in_profile(&listing_id, &ctx.profile_key)
        .or_else(|_| db.get_listed_item_by_id(&listing_id))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Listing not found".to_string())?;

    if item.status.as_deref() != Some("listed") {
        return Err("Listing is no longer active".into());
    }

    // Read save folder
    let save_folder = db
        .get_config("save_folder")
        .ok()
        .flatten()
        .unwrap_or_default();

    drop(db); // release lock before file IO

    let stash_path = crate::services::StashService::resolve_stash_path(&save_folder)
        .ok_or_else(|| "Shared stash not found".to_string())?;

    // Restore items to stash
    let item_code = resolve_item_code(&item.name)
        .ok_or_else(|| format!("Unknown item: {}", item.name))?;
    crate::services::StashService::modify_stackable(&stash_path, item_code, item.quantity)?;

    // Mark listing as cancelled
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.mark_listing_cancelled(&listing_id, &ctx.profile_key)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all items currently listed for sale
#[tauri::command]
pub fn get_listed_items(
    state: State<AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ListedItemResult>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);
    let items = if !ctx.profile_key.is_empty() {
        db.get_listed_items_in_profile_paginated(&ctx.profile_key, limit, offset)
            .or_else(|_| db.get_listed_items_paginated(limit, offset))
            .map_err(|e| e.to_string())?
    } else {
        db.get_listed_items_paginated(limit, offset).map_err(|e| e.to_string())?
    };
    Ok(items.into_iter().map(|i| ListedItemResult {
        id: i.id,
        name: i.name,
        quantity: i.quantity,
        unit_price: i.unit_price,
        listed_at: i.listed_at,
        sell_after_seconds: i.sell_after_seconds,
        status: i.status,
        item_code: i.item_code,
        item_kind: i.item_kind,
        quality: i.quality,
        listed_by: i.listed_by,
    }).collect())
}

/// Response for listed items (serializable)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ListedItemResult {
    pub id: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price: i32,
    pub listed_at: Option<String>,
    pub sell_after_seconds: i64,
    pub status: Option<String>,
    /// 物品 4-char code,如 "r01" (El Rune)、"gcv" (Chipped Amethyst)
    /// Catalog 用它计算 quality border 与 rune tier 排序
    pub item_code: Option<String>,
    /// 物品 kind,"rune" / "gem" / "potion" / "key" / "essence" / "armor" / "weapon" / "shield"
    pub item_kind: Option<String>,
    /// 物品品质,"unique" / "set" / "rare" / "magic" / "normal"
    pub quality: Option<String>,
    /// 上架人(角色存档文件名,如 "EchoingStrike.d2s")。
    /// Catalog 列表项显示 "ECHOINGSTRIKE 上架"
    pub listed_by: Option<String>,
}

/// Get a price suggestion for an item
#[tauri::command]
pub fn get_price_suggestion(
    item_name: String,
    item_kind: Option<String>,
) -> Result<PriceSuggestionResult, String> {
    let suggestion = get_sell_price_suggestion(&item_name, item_kind.as_deref());
    Ok(PriceSuggestionResult {
        base_price: suggestion.base_price,
        suggested_price: suggestion.suggested_price,
        min_price: suggestion.min_price,
        max_price: suggestion.max_price,
        variation: suggestion.variation,
        has_reference: suggestion.has_reference,
    })
}

/// List an item for sale on the marketplace.
/// Deducts from stash, creates a timed listing.
#[tauri::command]
pub fn list_item(
    state: State<AppState>,
    item_name: String,
    item_code: String,
    item_kind: Option<String>,
    quantity: i32,
    unit_price: i64,
    stash_file: Option<String>,
) -> Result<(), String> {
    if quantity < 1 || unit_price < 1 {
        return Err("Invalid quantity or price".into());
    }

    // Use provided stash_file or find from config
    let stash_path = if let Some(ref path) = stash_file {
        path.clone()
    } else {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let save_folder = db.get_config("save_folder").ok().flatten().unwrap_or_default();
        drop(db);
        if save_folder.is_empty() {
            return Err("Save folder not configured".into());
        }
        let candidates = ["ModernSharedStashSoftCoreV2.d2i", "ModernSharedStashHardCoreV2.d2i", "SharedStashSoftCoreV2.d2i", "SharedStashHardCoreV2.d2i"];
        let mut found = None;
        for f in &candidates {
            let p = std::path::Path::new(&save_folder).join(f);
            if p.exists() { found = Some(p.to_string_lossy().to_string()); break; }
        }
        found.ok_or_else(|| "Shared stash not found".to_string())?
    };

    // Deduct items from stash
    crate::services::StashService::modify_stackable(&stash_path, &item_code, -quantity)?;

    // Get reference price and calculate sell time
    let reference_price = get_market_reference_price(&item_name, item_kind.as_deref());
    let sell_after = calculate_sell_after_seconds(unit_price, reference_price, item_kind.as_deref());

    // Create listing record
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);
    let item_id = uuid::Uuid::new_v4().to_string();

    let listing = crate::database::VirtualItem {
        id: item_id.clone(),
        name: item_name.clone(),
        item_code: Some(item_code),
        item_kind,
        item_type: None,
        quality: None,
        level: None,
        attributes: None,
        source: Some("listed".into()),
        exported_from: None,
        purchased_at: None,
        token_price: Some(unit_price),
        status: Some("listed".into()),
        quantity: Some(quantity as i64),
        unit_price: Some(unit_price),
        listed_at: Some(chrono::Utc::now().to_rfc3339()),
        sell_after_seconds: Some(sell_after),
        profile_id: Some(ctx.profile_id),
        profile_key: Some(ctx.profile_key),
        game_version: Some(ctx.game_version),
        mod_name: Some(ctx.active_mod),
    };

    db.add_virtual_item(&listing).map_err(|e| e.to_string())?;
    db.add_transaction(
        "list",
        Some(&item_id),
        0,
        &format!("Listed {} x{} for {} tokens each", item_name, quantity, unit_price),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// 调整在售物品的单价(改价功能)。
/// 仅 status='listed' 的物品可改价。
/// 返回 Err 表示参数无效, Ok(true) 表示改价成功, Ok(false) 表示物品已下架。
#[tauri::command]
pub fn update_listing_price(
    state: State<AppState>,
    item_id: String,
    new_unit_price: i64,
) -> Result<bool, String> {
    if new_unit_price < 1 {
        return Err("New price must be at least 1 token".into());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ctx = get_market_profile_context(&db);

    // 读旧价 + 物品名, 用于记账
    let item = db.get_listed_item_by_id_in_profile(&item_id, &ctx.profile_key)
        .or_else(|_| db.get_listed_item_by_id(&item_id))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Item not found or not listed".to_string())?;
    let old_price = item.unit_price;

    let updated = db.update_listing_price(&item_id, new_unit_price, &ctx.profile_key)
        .map_err(|e| e.to_string())?;
    if !updated {
        return Ok(false);
    }

    // 写 transaction (token_amount=0, 仅记账事件)
    db.add_transaction(
        "reprice",
        Some(&item_id),
        0,
        &format!("Repriced {} from {} to {} tokens each", item.name, old_price, new_unit_price),
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_item_code ──

    #[test]
    fn test_resolve_el_rune() {
        assert_eq!(resolve_item_code("El Rune"), Some("r01"));
    }

    #[test]
    fn test_resolve_zod_rune() {
        assert_eq!(resolve_item_code("Zod Rune"), Some("r33"));
    }

    #[test]
    fn test_resolve_perfect_skull() {
        assert_eq!(resolve_item_code("Perfect Skull"), Some("skz"));
    }

    #[test]
    fn test_resolve_unknown_item() {
        assert_eq!(resolve_item_code("Fake Item Does Not Exist"), None);
    }

    #[test]
    fn test_resolve_case_insensitive() {
        assert_eq!(resolve_item_code("el rune"), Some("r01"));
        assert_eq!(resolve_item_code("EL RUNE"), Some("r01"));
    }

    #[test]
    fn test_resolve_ring() {
        assert_eq!(resolve_item_code("Ring"), Some("rin"));
    }

    #[test]
    fn test_resolve_amulet() {
        assert_eq!(resolve_item_code("Amulet"), Some("amu"));
    }

    #[test]
    fn test_resolve_empty_string() {
        assert_eq!(resolve_item_code(""), None);
    }
}
