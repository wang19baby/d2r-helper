//! Repository layer: domain-specific data access wrappers.
//!
//! Each repo wraps a `&rusqlite::Connection` and exposes
//! methods for a single bounded domain. This replaces the
//! monolithic `Database` god-struct pattern.
//!
//! Usage: `let repos = db.repos(); repos.market.get_balance()?`

pub mod config_repo;
pub mod market_repo;
pub mod traits;
pub mod warehouse_repo;

use rusqlite::Connection;

/// Aggregates all repositories for a single connection.
pub struct Repositories<'a> {
    pub config: config_repo::ConfigRepo<'a>,
    pub market: market_repo::MarketRepo<'a>,
    pub warehouse: warehouse_repo::WarehouseRepo<'a>,
}

impl DatabaseReposExt for Connection {
    fn repos(&self) -> Repositories<'_> {
        Repositories {
            config: config_repo::ConfigRepo::new(self),
            market: market_repo::MarketRepo::new(self),
            warehouse: warehouse_repo::WarehouseRepo::new(self),
        }
    }
}

/// Extension trait: add `.repos()` to any `&Connection`.
pub trait DatabaseReposExt {
    fn repos(&self) -> Repositories<'_>;
}
