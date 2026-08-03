use rusqlite::{Connection, Result, params};

use crate::database::repository::traits::ConfigRepository;

/// Configuration key-value store + resource profile queries.
pub struct ConfigRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ConfigRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT value FROM app_config WHERE key = ?1"
        )?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

impl ConfigRepository for ConfigRepo<'_> {
    fn get_config(&self, key: &str) -> Result<Option<String>, String> {
        self.get_config(key).map_err(|e| e.to_string())
    }
    fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        self.set_config(key, value).map_err(|e| e.to_string())
    }
}
