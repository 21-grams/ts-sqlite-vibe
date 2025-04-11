/// Database schema constants and helpers

/// Schema version
pub const SCHEMA_VERSION: i32 = 1;

/// SQL to create the initial schema tables and indices
pub const INITIAL_SCHEMA: &str = include_str!("../../migrations/001_initial_schema.sql");

/// SQLite pragmas for performance optimization
pub const PERFORMANCE_PRAGMAS: &str = r#"
-- Enable WAL mode for better concurrency and crash recovery
PRAGMA journal_mode = WAL;

-- Synchronous mode: NORMAL provides a good balance of safety and performance
PRAGMA synchronous = NORMAL;

-- Enable foreign keys for data integrity
PRAGMA foreign_keys = ON;

-- Increase cache size for better performance (10MB)
PRAGMA cache_size = 10000;

-- Increase page size for better performance with large datasets
PRAGMA page_size = 4096;

-- Optimize for time-series data access patterns
PRAGMA temp_store = MEMORY;
"#;

/// Get the SQL to optimize a time-series database
pub fn get_optimization_sql() -> String {
    r#"
-- Analyze the database to optimize query planning
ANALYZE;

-- Optimize the database (this is a no-op in SQLite but we include it for clarity)
PRAGMA optimize;

-- Vacuum the database to reclaim space from deleted records
-- Only run this occasionally as it rewrites the entire database
PRAGMA auto_vacuum = FULL;
VACUUM;
    "#.to_string()
}

/// Get the SQL to create indices for efficient time-series queries
pub fn get_time_series_indices_sql(table_name: &str, time_col: &str, id_col: &str) -> String {
    format!(
        r#"
-- Create index for time range queries
CREATE INDEX IF NOT EXISTS idx_{table_name}_{time_col} ON {table_name}({time_col});

-- Create composite index for efficient time series queries by ID and time
CREATE INDEX IF NOT EXISTS idx_{table_name}_{id_col}_{time_col} ON {table_name}({id_col}, {time_col});
        "#
    )
}

/// Get the SQL to run database maintenance tasks
pub fn get_maintenance_sql() -> String {
    r#"
-- Analyze database for query optimization
ANALYZE;

-- Clean up the WAL file
PRAGMA wal_checkpoint(FULL);

-- Run integrity check
PRAGMA integrity_check;
    "#.to_string()
}