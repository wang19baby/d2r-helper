/// Filters for warehouse_search — only non-None fields are applied.
#[derive(Debug, Clone, Default)]
pub struct WarehouseFilters {
    pub profile_key: Option<String>,
    pub source_character: Option<String>,
    pub item_kind: Option<String>,
    pub equipment_slot: Option<String>,
    pub quality: Option<String>,
    pub search_text: Option<String>,
    pub page_name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

use rusqlite::{Connection, Result, params};

use crate::database::models::WarehousedItem;

/// Warehouse domain: stashed item CRUD.
pub struct WarehouseRepo<'a> {
    conn: &'a Connection,
}

impl<'a> WarehouseRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn add(&self, item: &WarehousedItem) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO warehouse_items (
                id, item_code, item_name, item_kind, quality,
                simple_item, quantity, profile_key, game_version, mod_name,
                raw_item_bits, raw_bit_length, item_json,
                stash_name, imported_at, page_name, tags, notes,
                source_character, source_save_path, slot_equipped
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)"#,
            params![
                item.id, item.item_code, item.item_name, item.item_kind, item.quality,
                item.simple_item, item.quantity, item.profile_key, item.game_version, item.mod_name,
                item.raw_item_bits, item.raw_bit_length, item.item_json,
                item.stash_name, item.imported_at, item.page_name, item.tags, item.notes,
                item.source_character, item.source_save_path, item.slot_equipped,
            ],
        )?;
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items ORDER BY imported_at DESC"
        )?;
        let rows = stmt.query_map([], Self::map_row)?;
        rows.collect()
    }

    pub fn list_by_context(&self, mod_name: &str, game_version: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE mod_name = ?1 AND game_version = ?2 ORDER BY imported_at DESC"
        )?;
        let rows = stmt.query_map(params![mod_name, game_version], Self::map_row)?;
        rows.collect()
    }

    pub fn list_by_profile(&self, profile_key: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE profile_key = ?1 ORDER BY imported_at DESC"
        )?;
        let rows = stmt.query_map(params![profile_key], Self::map_row)?;
        rows.collect()
    }

    pub fn list_by_page(&self, page_name: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE page_name = ?1 ORDER BY imported_at DESC"
        )?;
        let rows = stmt.query_map(params![page_name], Self::map_row)?;
        rows.collect()
    }

    pub fn list_by_page_in_profile(&self, profile_key: &str, page_name: &str) -> Result<Vec<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE profile_key = ?1 AND page_name = ?2 ORDER BY imported_at DESC"
        )?;
        let rows = stmt.query_map(params![profile_key, page_name], Self::map_row)?;
        rows.collect()
    }

    pub fn get(&self, item_id: &str) -> Result<Option<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_inner(row)?)),
            None => Ok(None),
        }
    }

    pub fn get_in_profile(&self, profile_key: &str, item_id: &str) -> Result<Option<WarehousedItem>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM warehouse_items WHERE profile_key = ?1 AND id = ?2"
        )?;
        let mut rows = stmt.query(params![profile_key, item_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_inner(row)?)),
            None => Ok(None),
        }
    }

    pub fn remove(&self, item_id: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "DELETE FROM warehouse_items WHERE id = ?1",
            params![item_id],
        )?;
        Ok(updated > 0)
    }

    pub fn remove_in_profile(&self, profile_key: &str, item_id: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "DELETE FROM warehouse_items WHERE profile_key = ?1 AND id = ?2",
            params![profile_key, item_id],
        )?;
        Ok(updated > 0)
    }

    pub fn list_pages(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT DISTINCT page_name FROM warehouse_items ORDER BY page_name"
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    pub fn list_pages_in_profile(&self, profile_key: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT DISTINCT page_name FROM warehouse_items WHERE profile_key = ?1 ORDER BY page_name"
        )?;
        let rows = stmt.query_map(params![profile_key], |row| row.get(0))?;
        rows.collect()
    }


    /// Unified warehouse search — only non-None filters are applied.
    pub fn search(&self, filters: &WarehouseFilters) -> Result<Vec<WarehousedItem>> {
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref val) = filters.profile_key {
            conditions.push(format!("profile_key = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }
        if let Some(ref val) = filters.source_character {
            conditions.push(format!("source_character = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }
        if let Some(ref val) = filters.item_kind {
            conditions.push(format!("item_kind = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }
        if let Some(ref val) = filters.equipment_slot {
            conditions.push(format!("slot_equipped = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }
        if let Some(ref val) = filters.quality {
            conditions.push(format!("quality = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }
        if let Some(ref val) = filters.search_text {
            conditions.push(format!("item_name LIKE ?{}", param_values.len() + 1));
            param_values.push(Box::new(format!("%{}%", val)));
        }
        if let Some(ref val) = filters.page_name {
            conditions.push(format!("page_name = ?{}", param_values.len() + 1));
            param_values.push(Box::new(val.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit_clause = if let Some(limit) = filters.limit {
            let offset = filters.offset.unwrap_or(0);
            format!(" LIMIT {} OFFSET {}", limit, offset)
        } else {
            String::new()
        };

        let sql = format!("SELECT * FROM warehouse_items {} ORDER BY imported_at DESC{}", where_clause, limit_clause);
        let mut stmt = self.conn.prepare_cached(&sql)?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), Self::map_row)?;
        rows.collect()
    }

    pub fn update_meta(&self, item_id: &str, page_name: &str, tags: &str, notes: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE warehouse_items SET page_name = ?1, tags = ?2, notes = ?3 WHERE id = ?4",
            params![page_name, tags, notes, item_id],
        )?;
        Ok(updated > 0)
    }


    pub fn update_meta_in_profile(&self, profile_key: &str, item_id: &str, page_name: &str, tags: &str, notes: &str) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE warehouse_items SET page_name = ?1, tags = ?2, notes = ?3 WHERE profile_key = ?4 AND id = ?5",
            params![page_name, tags, notes, profile_key, item_id],
        )?;
        Ok(updated > 0)
    }

    fn map_inner(row: &rusqlite::Row<'_>) -> rusqlite::Result<WarehousedItem> {
        Ok(WarehousedItem {
            id: row.get(0)?,
            item_code: row.get(1)?,
            item_name: row.get(2)?,
            item_kind: row.get(3)?,
            quality: row.get(4)?,
            simple_item: row.get(5)?,
            quantity: row.get(6)?,
            profile_key: row.get(7)?,
            game_version: row.get(8)?,
            mod_name: row.get(9)?,
            raw_item_bits: row.get(10)?,
            raw_bit_length: row.get(11)?,
            item_json: row.get(12)?,
            stash_name: row.get(13)?,
            imported_at: row.get(14)?,
            page_name: row.get(15)?,
            tags: row.get(16)?,
            notes: row.get(17)?,
            source_character: row.get(18)?,
            source_save_path: row.get(19)?,
            slot_equipped: row.get(20)?,
            page_index: row.get(21)?,
            position_x: row.get(22)?,
            position_y: row.get(23)?,
            inv_width: row.get(24)?,
            inv_height: row.get(25)?,
        })
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WarehousedItem> {
        Self::map_inner(row)
    }
}
