use axum::{
    extract::Query,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::get_connection;
use crate::utils::error::AppError;

#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub database_size_mb: f64,
    pub free_space_mb: f64,
    pub last_backup: Option<i64>,
    pub readings_count: i64,
    pub oldest_reading: Option<i64>,
    pub newest_reading: Option<i64>,
    pub average_insert_rate: Option<f64>,
    pub peak_insert_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceRequest {
    pub tasks: Vec<String>,
    pub archive_before: Option<i64>,
}

/// Get the health status of the database
pub async fn get_database_health() -> Result<Json<DatabaseHealth>, AppError> {
    let conn = get_connection()?;
    
    // Get database size
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "sensor_data.db".to_string());
    let path = Path::new(&db_path);
    
    let db_size = match path.metadata() {
        Ok(metadata) => metadata.len() as f64 / (1024.0 * 1024.0), // Convert to MB
        Err(_) => -1.0, // Unable to get file size
    };
    
    // Get free disk space (platform-specific)
    let free_space = if cfg!(unix) {
        #[cfg(unix)]
        {
            use std::fs;
            let parent_dir = path.parent().unwrap_or_else(|| Path::new("/"));
            match fs::statvfs(parent_dir) {
                Ok(stat) => {
                    let free_bytes = stat.blocks_free() * stat.block_size();
                    free_bytes as f64 / (1024.0 * 1024.0) // Convert to MB
                }
                Err(_) => -1.0,
            }
        }
        #[cfg(not(unix))]
        {
            -1.0 // Not implemented for non-Unix platforms
        }
    } else {
        -1.0 // Not implemented for non-Unix platforms
    };
    
    // Get readings count
    let readings_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM readings",
        [],
        |row| row.get(0),
    )?;
    
    // Get oldest reading
    let oldest_reading: Option<i64> = if readings_count > 0 {
        conn.query_row(
            "SELECT MIN(timestamp) FROM readings",
            [],
            |row| row.get(0),
        ).ok()
    } else {
        None
    };
    
    // Get newest reading
    let newest_reading: Option<i64> = if readings_count > 0 {
        conn.query_row(
            "SELECT MAX(timestamp) FROM readings",
            [],
            |row| row.get(0),
        ).ok()
    } else {
        None
    };
    
    // Calculate insertion rates (if possible)
    let (average_insert_rate, peak_insert_rate) = if let (Some(oldest), Some(newest)) = (oldest_reading, newest_reading) {
        let duration_seconds = (newest - oldest) as f64;
        
        if duration_seconds > 0.0 {
            let avg_rate = readings_count as f64 / duration_seconds;
            
            // Calculate peak rate (based on hourly windows)
            let peak_rate: f64 = conn.query_row(
                "SELECT MAX(count) FROM (
                    SELECT COUNT(*) as count 
                    FROM readings 
                    GROUP BY timestamp / 3600
                )",
                [],
                |row| row.get(0),
            ).unwrap_or(0.0);
            
            (Some(avg_rate), Some(peak_rate / 3600.0)) // Convert hourly peak to per-second
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    
    // Determine status
    let status = if readings_count > 0 && newest_reading.is_some() {
        "healthy"
    } else {
        "empty"
    };
    
    let health = DatabaseHealth {
        status: status.to_string(),
        database_size_mb: db_size,
        free_space_mb: free_space,
        last_backup: None, // Not implemented
        readings_count,
        oldest_reading,
        newest_reading,
        average_insert_rate,
        peak_insert_rate,
    };
    
    Ok(Json(health))
}

/// Run database maintenance tasks
pub async fn run_maintenance(
    Json(payload): Json<MaintenanceRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let mut conn = get_connection()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let mut tasks_completed = Vec::new();
    let mut archive_count = 0;
    let start_time = std::time::Instant::now();
    
    // Begin transaction
    let mut tx = conn.transaction()?;
    
    for task in &payload.tasks {
        match task.as_str() {
            "analyze" => {
                tx.execute("ANALYZE", [])?;
                tasks_completed.push("analyze");
            },
            "optimize" => {
                tx.execute("PRAGMA optimize", [])?;
                tasks_completed.push("optimize");
            },
            "vacuum" => {
                // Note: VACUUM cannot be executed within a transaction
                tasks_completed.push("vacuum");
            },
            _ => {
                // Skip unknown tasks
            }
        }
    }
    
    // Archive old readings if requested
    if let Some(archive_before) = payload.archive_before {
        let deleted = tx.execute(
            "DELETE FROM readings WHERE timestamp < ?",
            [archive_before],
        )?;
        
        archive_count = deleted;
    }
    
    // Commit transaction
    tx.commit()?;
    
    // Run VACUUM outside the transaction if requested
    if payload.tasks.contains(&"vacuum".to_string()) {
        conn.execute("VACUUM", [])?;
    }
    
    // Calculate new database size
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "sensor_data.db".to_string());
    let path = Path::new(&db_path);
    
    let new_db_size = match path.metadata() {
        Ok(metadata) => metadata.len() as f64 / (1024.0 * 1024.0), // Convert to MB
        Err(_) => -1.0, // Unable to get file size
    };
    
    let elapsed = start_time.elapsed().as_secs_f64();
    
    let response = json!({
        "success": true,
        "tasks_completed": tasks_completed,
        "archived_readings": archive_count,
        "duration_seconds": elapsed,
        "new_database_size_mb": new_db_size
    });
    
    Ok((StatusCode::OK, Json(response)))
}

/// Export sensor data
pub async fn export_data(
    Query(query): Query<ExportQuery>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // This is a placeholder for a real implementation
    // In a complete implementation, we would:
    // 1. Query the readings based on the parameters
    // 2. Convert to the requested format
    // 3. Return a file download response
    
    let response = json!({
        "message": "Export functionality not fully implemented in this example",
        "params": {
            "sensor_ids": query.sensor_ids,
            "start_time": query.start_time,
            "end_time": query.end_time,
            "format": query.format
        }
    });
    
    Ok((StatusCode::OK, Json(response)))
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub sensor_ids: Option<String>, // Comma-separated list
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub format: Option<String>, // 'json', 'csv', 'excel'
}