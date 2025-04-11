# SQLite Performance Tuning for Time-Series Data

This document outlines the performance optimizations used in this project to make SQLite efficient for time-series data from sensors.

## Database Configuration

### WAL Mode

We use Write-Ahead Logging (WAL) mode for better concurrency and performance:

```sql
PRAGMA journal_mode = WAL;
```

Benefits:
- Better concurrency (readers don't block writers and vice versa)
- Faster transaction commit times
- Better crash recovery

### Synchronous Mode

We use NORMAL synchronous mode for a good balance of safety and performance:

```sql
PRAGMA synchronous = NORMAL;
```

Benefits:
- Reduces disk I/O while maintaining reasonable safety
- Approximately 2-3x faster than FULL mode

### Cache Size

We increase the cache size to improve performance:

```sql
PRAGMA cache_size = 10000; -- approximately 10MB
```

Benefits:
- Reduced disk I/O by keeping more data in memory
- Improved query performance for recently accessed data

### Page Size

We optimize the page size for time-series data:

```sql
PRAGMA page_size = 4096;
```

Benefits:
- Better alignment with filesystem blocks on most systems
- More efficient storage for time-series data

## Schema Optimizations

### Indexing Strategy

1. **Composite Index for Time Range Queries by Sensor**:
   ```sql
   CREATE INDEX idx_readings_sensor_time ON readings(sensor_id, timestamp);
   ```
   - Optimizes queries filtering by sensor_id and a time range
   - Essential for the most common query pattern

2. **Simple Index for Global Time Range Queries**:
   ```sql
   CREATE INDEX idx_readings_timestamp ON readings(timestamp);
   ```
   - Optimizes queries across all sensors in a time range

### Table Structure

1. **Separation of Metadata and Time-Series Data**:
   - `sensors` table for metadata (updated infrequently)
   - `readings` table for time-series data (append-mostly)

2. **Minimal Schema for Readings**:
   - Only essential columns to minimize row size
   - Support for both analog (value) and digital (state) readings

### Efficient Data Loading

1. **Transaction Batching**:
   - Bulk inserts wrapped in a single transaction
   - Significantly reduces overhead for large imports

2. **Prepared Statements**:
   - Reuse of prepared statements for repeated operations
   - Reduces SQL parsing overhead

## Query Optimization

### Efficient Time Range Filtering

```sql
SELECT * FROM readings 
WHERE sensor_id = ? AND timestamp >= ? AND timestamp <= ?
ORDER BY timestamp
```

- Uses the composite index efficiently
- Minimizes full table scans

### Current Reading Optimization

```sql
SELECT * FROM readings 
WHERE sensor_id = ? 
ORDER BY timestamp DESC 
LIMIT 1
```

- Uses the composite index efficiently
- LIMIT 1 terminates the search early

### Aggregation Optimization

For aggregated time-series data:

```sql
SELECT 
  (timestamp / ?) * ? as period_start, 
  AVG(value) as avg_value,
  MIN(value) as min_value,
  MAX(value) as max_value,
  COUNT(*) as sample_count
FROM readings
WHERE sensor_id = ? AND timestamp >= ? AND timestamp <= ?
GROUP BY period_start
ORDER BY period_start
```

- Integer division for efficient time bucketing
- Uses the composite index for the WHERE clause

## Maintenance

### Regular ANALYZE

We periodically run ANALYZE to update statistics for the query planner:

```sql
ANALYZE;
```

### WAL Checkpointing

We manage the WAL file size with regular checkpoints:

```sql
PRAGMA wal_checkpoint(FULL);
```

### Integrity Checks

We periodically check database integrity:

```sql
PRAGMA integrity_check;
```

## Connection Pooling

We use r2d2 connection pooling to:
- Reuse database connections efficiently
- Manage the maximum number of concurrent connections
- Handle connection timeouts and circuit breaking

## Monitoring

Key metrics to monitor:
- Query execution times
- Transaction durations
- Cache hit rates
- WAL file size
- Database file size growth

## Partitioning Strategy

For very large datasets (billions of readings), consider:

1. **Time-based partitioning**:
   - Split readings into separate tables by time period (e.g., monthly tables)
   - Use a view to unify queries across partitions

2. **Sensor-based partitioning**:
   - For installations with thousands of sensors
   - Group related sensors into separate tables

## Recommended SQLite Version

Use SQLite 3.35.0 or higher for:
- Better query planner
- Improved WAL mode performance
- Support for materialized CTEs
- Enhanced window functions

## Compression

SQLite does not natively compress data, but consider:
- Hardware compression (ZFS, Btrfs)
- Row-level compression for text fields
- External compression for backups