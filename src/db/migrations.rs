use anyhow::{Context, Result};
use rusqlite::Connection;

/// Schema version
const CURRENT_VERSION: i32 = 1;

/// Run database migrations
pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    // Create schema_version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )
    .context("Failed to create schema_version table")?;

    // Get current schema version
    let version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < CURRENT_VERSION {
        // Begin transaction for migration
        let tx = conn.transaction().context("Failed to begin transaction")?;

        if version == 0 {
            // Initial schema
            tx.execute_batch(include_str!("../../migrations/001_initial_schema.sql"))
                .context("Failed to apply initial schema migration")?;
        }

        // Update schema version
        tx.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            [CURRENT_VERSION],
        )
        .context("Failed to update schema version")?;

        // Commit transaction
        tx.commit().context("Failed to commit migration transaction")?;
    }

    Ok(())
}