use rusqlite::{Connection, Result, params};

use crate::database::models::{
    ListedItem, SoldItem, Transaction, VirtualItem,
};
use crate::database::repository::traits::MarketRepository;
use crate::market::pricing::calculate_sell_price;

/// Market domain: tokens, listings, transactions, virtual items.
pub struct MarketRepo<'a> {
    conn: &'a Connection,
}

impl<'a> MarketRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Token balance ──

    pub fn get_token_balance(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT COALESCE(SUM(amount), 0) FROM token_balance"
        )?;
        stmt.query_row([], |row| row.get(0))
    }

    pub fn update_token_balance(&self, amount: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO token_balance (amount) VALUES (?1)",
            params![amount],
        )?;
        Ok(())
    }

    // ── Listings ──

    pub fn get_listed_items(&self) -> Result<Vec<ListedItem>> {
        let sql = r#"
            SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                   vi.listed_at, vi.sell_after_seconds, vi.status,
                   vi.item_code, vi.item_kind, vi.quality,
                   vi.exported_from AS listed_by
            FROM virtual_items vi
            WHERE vi.status = 'listed'
            ORDER BY vi.listed_at DESC
        "#;
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], Self::map_listed_item)?;
        rows.collect()
    }

    pub fn get_listed_items_in_profile(&self, profile_key: &str) -> Result<Vec<ListedItem>> {
        let mut stmt = self.conn.prepare_cached(
            r#"SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                      vi.listed_at, vi.sell_after_seconds, vi.status,
                      vi.item_code, vi.item_kind, vi.quality,
                      vi.exported_from AS listed_by
               FROM virtual_items vi
               WHERE vi.status = 'listed' AND vi.profile_key = ?1
               ORDER BY vi.listed_at DESC"#
        )?;
        let rows = stmt.query_map(params![profile_key], Self::map_listed_item)?;
        rows.collect()
    }

    pub fn get_listed_items_paginated(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>> {
        let limit_clause = match limit {
            Some(l) => format!(" LIMIT {}", l),
            None => String::new(),
        };
        let offset_clause = match offset {
            Some(o) => format!(" OFFSET {}", o),
            None => String::new(),
        };
        let sql = format!(
            "SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                    vi.listed_at, vi.sell_after_seconds, vi.status,
                    vi.item_code, vi.item_kind, vi.quality,
                    vi.exported_from AS listed_by
             FROM virtual_items vi
             WHERE vi.status = 'listed'
             ORDER BY vi.listed_at DESC{}{}",
            limit_clause, offset_clause
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map([], Self::map_listed_item)?;
        rows.collect()
    }

    pub fn get_listed_items_in_profile_paginated(&self, profile_key: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>> {
        let limit_clause = match limit {
            Some(l) => format!(" LIMIT {}", l),
            None => String::new(),
        };
        let offset_clause = match offset {
            Some(o) => format!(" OFFSET {}", o),
            None => String::new(),
        };
        let sql = format!(
            "SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                    vi.listed_at, vi.sell_after_seconds, vi.status,
                    vi.item_code, vi.item_kind, vi.quality,
                    vi.exported_from AS listed_by
             FROM virtual_items vi
             WHERE vi.status = 'listed' AND vi.profile_key = ?1
             ORDER BY vi.listed_at DESC{}{}",
            limit_clause, offset_clause
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![profile_key], Self::map_listed_item)?;
        rows.collect()
    }

    pub fn get_listed_item_by_id(&self, item_id: &str) -> Result<Option<ListedItem>> {
        let sql = r#"
            SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                   vi.listed_at, vi.sell_after_seconds, vi.status,
                   vi.item_code, vi.item_kind, vi.quality,
                   vi.exported_from AS listed_by
            FROM virtual_items vi
            WHERE vi.id = ?1
        "#;
        let mut stmt = self.conn.prepare_cached(sql)?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_listed_item_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn get_listed_item_by_id_in_profile(&self, item_id: &str, profile_key: &str) -> Result<Option<ListedItem>> {
        let sql = r#"
            SELECT vi.id, vi.name, vi.quantity, vi.unit_price,
                   vi.listed_at, vi.sell_after_seconds, vi.status,
                   vi.item_code, vi.item_kind, vi.quality,
                   vi.exported_from AS listed_by
            FROM virtual_items vi
            WHERE vi.id = ?1 AND vi.profile_key = ?2
        "#;
        let mut stmt = self.conn.prepare_cached(sql)?;
        let mut rows = stmt.query(params![item_id, profile_key])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_listed_item_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn mark_listing_cancelled(&self, item_id: &str, profile_key: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE virtual_items SET status = 'cancelled' WHERE id = ?1 AND profile_key = ?2",
            params![item_id, profile_key],
        )?;
        Ok(updated > 0)
    }

    pub fn update_listing_price(&self, item_id: &str, new_unit_price: i64, profile_key: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE virtual_items SET unit_price = ?1 WHERE id = ?2 AND profile_key = ?3 AND status = 'listed'",
            params![new_unit_price, item_id, profile_key],
        )?;
        Ok(updated > 0)
    }

    pub fn mark_listing_sold(&self, item_id: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE virtual_items SET status = 'sold' WHERE id = ?1",
            params![item_id],
        )?;
        Ok(updated > 0)
    }

    pub fn mark_item_as_sold(&self, item_id: &str, profile_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE virtual_items SET status = 'sold' WHERE id = ?1 AND profile_key = ?2",
            params![item_id, profile_key],
        )?;
        Ok(())
    }

    pub fn mark_item_as_imported(&self, item_id: &str, profile_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE virtual_items SET status = 'imported' WHERE id = ?1 AND profile_key = ?2",
            params![item_id, profile_key],
        )?;
        Ok(())
    }

    // ── Virtual items ──

    pub fn add_virtual_item(&self, item: &VirtualItem) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO virtual_items (
                id, name, item_code, item_kind, item_type, quality,
                level, attributes, source, exported_from, purchased_at,
                token_price, status, quantity, unit_price,
                listed_at, sell_after_seconds,
                profile_id, profile_key, game_version, mod_name
            ) VALUES (
                ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21
            )"#,
            params![
                item.id, item.name, item.item_code, item.item_kind, item.item_type,
                item.quality, item.level, item.attributes,
                item.source, item.exported_from, item.purchased_at,
                item.token_price, item.status, item.quantity, item.unit_price,
                item.listed_at, item.sell_after_seconds,
                item.profile_id, item.profile_key, item.game_version, item.mod_name,
            ],
        )?;
        Ok(())
    }

    pub fn get_virtual_items(&self, status: &str) -> Result<Vec<VirtualItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM virtual_items WHERE status = ?1 ORDER BY purchased_at DESC"
        )?;
        let rows = stmt.query_map(params![status], Self::map_virtual_item)?;
        rows.collect()
    }

    pub fn get_virtual_items_in_profile(&self, status: &str, profile_key: &str) -> Result<Vec<VirtualItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM virtual_items WHERE status = ?1 AND profile_key = ?2 ORDER BY purchased_at DESC"
        )?;
        let rows = stmt.query_map(params![status, profile_key], Self::map_virtual_item)?;
        rows.collect()
    }

    pub fn get_virtual_item_by_id(&self, item_id: &str) -> Result<Option<VirtualItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM virtual_items WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_virtual_item_row(row)?)),
            None => Ok(None),
        }
    }

    // ── Transactions ──

    pub fn add_transaction(&self, tx_type: &str, item_id: Option<&str>, amount: i64, description: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO transactions (tx_type, item_id, token_amount, description) VALUES (?1, ?2, ?3, ?4)",
            params![tx_type, item_id, amount, description],
        )?;
        Ok(())
    }

    pub fn get_transactions(&self, limit: i64, tx_type: Option<&str>) -> Result<Vec<Transaction>> {
        let (sql, has_filter) = if let Some(t) = tx_type {
            (format!(
                "SELECT * FROM transactions WHERE tx_type = '{}' ORDER BY date DESC LIMIT ?1", t
            ), true)
        } else {
            ("SELECT * FROM transactions ORDER BY date DESC LIMIT ?1".to_string(), false)
        };
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = if has_filter {
            stmt.query_map(params![limit], Self::map_transaction)?
        } else {
            stmt.query_map(params![limit], Self::map_transaction)?
        };
        rows.collect()
    }

    pub fn process_due_listings(&self) -> Result<Vec<SoldItem>> {
        let mut stmt = self.conn.prepare_cached(
            r#"SELECT id, name, quantity, unit_price, listed_at, sell_after_seconds
               FROM virtual_items
               WHERE status = 'listed'
                 AND (CAST(strftime('%s', 'now') AS INTEGER) -
                      CAST(strftime('%s', listed_at) AS INTEGER)) >= sell_after_seconds
               LIMIT 50"#
        )?;
        let sold: Vec<SoldItem> = stmt.query_map([], |row| {
            Ok(SoldItem {
                id: row.get(0)?,
                name: row.get(1)?,
                quantity: row.get(2)?,
                unit_price: row.get(3)?,
                listed_at: row.get(4)?,
                sell_after_seconds: row.get(5)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        for item in &sold {
            let token_price = item.unit_price as i64;
            let sell_price = calculate_sell_price(token_price);
            self.conn.execute(
                "UPDATE virtual_items SET status = 'sold' WHERE id = ?1",
                params![item.id],
            )?;
            self.conn.execute(
                "INSERT INTO token_balance (amount) VALUES (?1)",
                params![sell_price],
            )?;
            self.conn.execute(
                "INSERT INTO transactions (tx_type, item_id, token_amount, description) VALUES (?1, ?2, ?3, ?4)",
                params!["auto_sell", &item.id, sell_price, format!("Auto-sold: {}x {}", item.quantity, item.name)],
            )?;
        }
        Ok(sold)
    }

    // ── Row mappers ──

    fn map_listed_item_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ListedItem> {
        Ok(ListedItem {
            id: row.get(0)?,
            name: row.get(1)?,
            quantity: row.get(2)?,
            unit_price: row.get(3)?,
            listed_at: row.get(4)?,
            sell_after_seconds: row.get(5)?,
            status: row.get(6)?,
            item_code: row.get(7)?,
            item_kind: row.get(8)?,
            quality: row.get(9)?,
            listed_by: row.get(10)?,
        })
    }

    fn map_listed_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ListedItem> {
        Self::map_listed_item_row(row)
    }

    fn map_virtual_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<VirtualItem> {
        Ok(VirtualItem {
            id: row.get(0)?,
            name: row.get(1)?,
            item_code: row.get(2)?,
            item_kind: row.get(3)?,
            item_type: row.get(4)?,
            quality: row.get(5)?,
            level: row.get(6)?,
            attributes: row.get(7)?,
            source: row.get(8)?,
            exported_from: row.get(9)?,
            purchased_at: row.get(10)?,
            token_price: row.get(11)?,
            status: row.get(12)?,
            quantity: row.get(13)?,
            unit_price: row.get(14)?,
            listed_at: row.get(15)?,
            sell_after_seconds: row.get(16)?,
            profile_id: row.get(17)?,
            profile_key: row.get(18)?,
            game_version: row.get(19)?,
            mod_name: row.get(20)?,
        })
    }

    fn map_virtual_item_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VirtualItem> {
        Self::map_virtual_item(row)
    }

    fn map_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
        Ok(Transaction {
            id: row.get(0)?,
            tx_type: row.get(1)?,
            item_id: row.get(2)?,
            token_amount: row.get(3)?,
            description: row.get(4)?,
            date: row.get(5)?,
        })
    }
}

impl MarketRepository for MarketRepo<'_> {
    fn get_token_balance(&self) -> Result<i64, String> {
        self.get_token_balance().map_err(|e| e.to_string())
    }
    fn update_token_balance(&self, amount: i64) -> Result<(), String> {
        self.update_token_balance(amount).map_err(|e| e.to_string())
    }
    fn add_virtual_item(&self, item: &VirtualItem) -> Result<(), String> {
        self.add_virtual_item(item).map_err(|e| e.to_string())
    }
    fn get_listed_items(&self) -> Result<Vec<ListedItem>, String> {
        self.get_listed_items().map_err(|e| e.to_string())
    }
    fn get_listed_items_in_profile(&self, profile_key: &str) -> Result<Vec<ListedItem>, String> {
        self.get_listed_items_in_profile(profile_key).map_err(|e| e.to_string())
    }
    fn get_listed_items_paginated(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>, String> {
        self.get_listed_items_paginated(limit, offset).map_err(|e| e.to_string())
    }
    fn get_listed_items_in_profile_paginated(&self, profile_key: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ListedItem>, String> {
        self.get_listed_items_in_profile_paginated(profile_key, limit, offset).map_err(|e| e.to_string())
    }
    fn get_listed_item_by_id(&self, item_id: &str) -> Result<Option<ListedItem>, String> {
        self.get_listed_item_by_id(item_id).map_err(|e| e.to_string())
    }
    fn get_listed_item_by_id_in_profile(&self, item_id: &str, profile_key: &str) -> Result<Option<ListedItem>, String> {
        self.get_listed_item_by_id_in_profile(item_id, profile_key).map_err(|e| e.to_string())
    }
    fn get_virtual_items(&self, status: &str) -> Result<Vec<VirtualItem>, String> {
        self.get_virtual_items(status).map_err(|e| e.to_string())
    }
    fn get_virtual_items_in_profile(&self, status: &str, profile_key: &str) -> Result<Vec<VirtualItem>, String> {
        self.get_virtual_items_in_profile(status, profile_key).map_err(|e| e.to_string())
    }
    fn get_virtual_item_by_id(&self, item_id: &str) -> Result<Option<VirtualItem>, String> {
        self.get_virtual_item_by_id(item_id).map_err(|e| e.to_string())
    }
    fn mark_listing_cancelled(&self, item_id: &str, profile_key: &str) -> Result<bool, String> {
        self.mark_listing_cancelled(item_id, profile_key).map_err(|e| e.to_string())
    }
    fn update_listing_price(&self, item_id: &str, new_unit_price: i64, profile_key: &str) -> Result<bool, String> {
        self.update_listing_price(item_id, new_unit_price, profile_key).map_err(|e| e.to_string())
    }
    fn mark_item_as_sold(&self, item_id: &str, profile_key: &str) -> Result<(), String> {
        self.mark_item_as_sold(item_id, profile_key).map_err(|e| e.to_string())
    }
    fn mark_item_as_imported(&self, item_id: &str, profile_key: &str) -> Result<(), String> {
        self.mark_item_as_imported(item_id, profile_key).map_err(|e| e.to_string())
    }
    fn add_transaction(&self, tx_type: &str, item_id: Option<&str>, amount: i64, description: &str) -> Result<(), String> {
        self.add_transaction(tx_type, item_id, amount, description).map_err(|e| e.to_string())
    }
    fn get_transactions(&self, limit: i64, tx_type: Option<&str>) -> Result<Vec<Transaction>, String> {
        self.get_transactions(limit, tx_type).map_err(|e| e.to_string())
    }
    fn process_due_listings(&self) -> Result<Vec<SoldItem>, String> {
        self.process_due_listings().map_err(|e| e.to_string())
    }
}
