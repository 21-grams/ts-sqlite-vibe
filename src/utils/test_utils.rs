#![cfg(test)]

use anyhow::Result;
use rusqlite::Connection;
use std::sync::Once;
use tempfile::TempDir;

use crate::db::{init_pool, migrations};

static INIT: Once = Once::new();

/// Initialize the in-memory test database
pub fn setup_test_db() -> Result<&'static r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>> {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("sensor_monitoring_api=debug")
            .try_init();
    });
    
    crate::db::init_test_pool()
}

/// Create a temporary database file for testing
pub fn setup_temp_db_file() -> Result<(TempDir, Connection)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    
    let conn = Connection::open(&db_path)?;
    migrations::run_migrations(&conn)?;
    
    Ok((temp_dir, conn))
}

/// Create a test sensor in the database
pub fn create_test_sensor(conn: &Connection) -> Result<i64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    
    conn.execute(
        "INSERT INTO sensors (
            sensor_name, sensor_type, location, unit, 
            threshold_min, threshold_max, notes,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            "Test Sensor",
            "temperature",
            "Test Location",
            "C",
            18.0,
            25.0,
            "Test Notes",
            now,
            now
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}

/// Create a test reading in the database
pub fn create_test_reading(conn: &Connection, sensor_id: i64) -> Result<i64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    
    conn.execute(
        "INSERT INTO readings (
            timestamp, sensor_id, value, change_type
        ) VALUES (?, ?, ?, ?)",
        rusqlite::params![
            now,
            sensor_id,
            21.5,
            "periodic"
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}

/// Create a test logging session in the database
pub fn create_test_session(conn: &Connection, sensor_id: i64, active: bool) -> Result<i64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    
    let end_time = if active { 
        None 
    } else { 
        Some(now + 3600) // 1 hour later
    };
    
    conn.execute(
        "INSERT INTO logging_sessions (
            sensor_id, start_time, end_time, sample_rate, notes
        ) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            sensor_id,
            now,
            end_time,
            300, // 5 minutes
            "Test Session"
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}