use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;

pub mod migrations;
pub mod schema;

type DbPool = Pool<SqliteConnectionManager>;
static DB_POOL: OnceCell<DbPool> = OnceCell::new();

/// Initialize the database connection pool
pub fn init_pool(db_path: &Path) -> Result<&'static DbPool> {
    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA cache_size = 10000;",
            )?;
            Ok(())
        });

    let pool = Pool::new(manager).context("Failed to create database connection pool")?;
    
    DB_POOL.get_or_init(|| pool);
    
    // Run migrations
    let conn = get_connection()?;
    migrations::run_migrations(&conn)?;

    Ok(DB_POOL.get().unwrap())
}

/// Get a connection from the pool
pub fn get_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
    match DB_POOL.get() {
        Some(pool) => Ok(pool.get().context("Failed to get database connection from pool")?),
        None => Err(anyhow::anyhow!("Database pool not initialized")),
    }
}

/// Get the database pool
pub fn get_pool() -> Result<&'static DbPool> {
    match DB_POOL.get() {
        Some(pool) => Ok(pool),
        None => Err(anyhow::anyhow!("Database pool not initialized")),
    }
}

/// Create a new in-memory database for testing
#[cfg(test)]
pub fn init_test_pool() -> Result<&'static DbPool> {
    let manager = SqliteConnectionManager::memory().with_init(|conn| {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(())
    });

    let pool = Pool::new(manager).context("Failed to create test database pool")?;
    
    DB_POOL.get_or_init(|| pool);
    
    // Run migrations on the test database
    let conn = get_connection()?;
    migrations::run_migrations(&conn)?;

    Ok(DB_POOL.get().unwrap())
}