//! Trait definitions for repository pattern (enables testing with mocks).

use crate::database::models::{
    ListedItem, SoldItem, Transaction, VirtualItem, WarehousedItem,
};

// ── ConfigRepo trait ──

pub trait ConfigRepository {
    fn get_config(&self, key: &str) -> Result<Option<String>, String>;
    fn set_config(&self, key: &str, value: &str) -> Result<(), String>;
}

// ── MarketRepo trait ──

pub trait MarketRepository {
    fn get_token_balance(&self) -> Result<i64, String>;
    fn update_token_balance(&self, amount: i64) -> Result<(), String>;
    fn add_virtual_item(&self, item: &VirtualItem) -> Result<(), String>;
    fn get_listed_items(&self) -> Result<Vec<ListedItem>, String>;
    fn get_listed_items_in_profile(&self, profile_key: &str) -> Result<Vec<ListedItem>, String>;
    fn get_listed_items_paginated(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>, String>;
    fn get_listed_items_in_profile_paginated(&self, profile_key: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>, String>;
    fn get_listed_item_by_id(&self, item_id: &str) -> Result<Option<ListedItem>, String>;
    fn get_listed_item_by_id_in_profile(&self, item_id: &str, profile_key: &str) -> Result<Option<ListedItem>, String>;
    fn get_virtual_items(&self, status: &str) -> Result<Vec<VirtualItem>, String>;
    fn get_virtual_items_in_profile(&self, status: &str, profile_key: &str) -> Result<Vec<VirtualItem>, String>;
    fn get_virtual_item_by_id(&self, item_id: &str) -> Result<Option<VirtualItem>, String>;
    fn mark_listing_cancelled(&self, item_id: &str, profile_key: &str) -> Result<bool, String>;
    fn update_listing_price(&self, item_id: &str, new_unit_price: i64, profile_key: &str) -> Result<bool, String>;
    fn mark_item_as_sold(&self, item_id: &str, profile_key: &str) -> Result<(), String>;
    fn mark_item_as_imported(&self, item_id: &str, profile_key: &str) -> Result<(), String>;
    fn add_transaction(&self, tx_type: &str, item_id: Option<&str>, amount: i64, description: &str) -> Result<(), String>;
    fn get_transactions(&self, limit: i64, tx_type: Option<&str>) -> Result<Vec<Transaction>, String>;
    fn process_due_listings(&self) -> Result<Vec<SoldItem>, String>;
}

// ── WarehouseRepo trait ──

pub trait WarehouseRepository {
    fn warehouse_add(&self, item: &WarehousedItem) -> Result<(), String>;
    fn warehouse_list_all(&self) -> Result<Vec<WarehousedItem>, String>;
    fn warehouse_list_by_profile(&self, profile_key: &str) -> Result<Vec<WarehousedItem>, String>;
    fn warehouse_list_by_page_in_profile(&self, profile_key: &str, page_name: &str) -> Result<Vec<WarehousedItem>, String>;
    fn warehouse_get_in_profile(&self, profile_key: &str, item_id: &str) -> Result<Option<WarehousedItem>, String>;
    fn warehouse_remove_in_profile(&self, profile_key: &str, item_id: &str) -> Result<bool, String>;
    fn warehouse_list_pages_in_profile(&self, profile_key: &str) -> Result<Vec<String>, String>;
    fn warehouse_update_meta_in_profile(&self, profile_key: &str, item_id: &str, page_name: &str, tags: &str, notes: &str) -> Result<bool, String>;
}
