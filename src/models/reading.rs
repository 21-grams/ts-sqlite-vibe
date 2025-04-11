use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::get_connection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub reading_id: Option<i64>,
    pub timestamp: Option<i64>,  // Will be set automatically if not provided
    pub sensor_id: i64,
    pub value: Option<f64>,      // For analog sensors
    pub state: Option<i64>,      // For digital/boolean sensors
    pub change_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingResponse {
    pub reading_id: i64,
    pub timestamp: DateTime<Utc>,
    pub sensor_id: i64,
    pub value: Option<f64>,
    pub state: Option<i64>,
    pub change_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingQuery {
    pub sensor_id: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingBulkInsert {
    pub readings: Vec<Reading>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingBulkResponse {
    pub inserted_count: usize,
    pub success: bool,
}

impl Reading {
    /// Create a new reading
    pub fn create(&self) -> Result<i64> {
        let conn = get_connection()?;
        
        // Use current time if timestamp is not provided
        let timestamp = self.timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64
        });
        
        let result = conn.execute(
            "INSERT INTO readings (
                timestamp, sensor_id, value, state, change_type
            ) VALUES (?, ?, ?, ?, ?)",
            params![
                timestamp,
                self.sensor_id,
                self.value,
                self.state,
                self.change_type
            ],
        )?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to create reading"));
        }
        
        let id = conn.last_insert_rowid();
        Ok(id)
    }
    
    /// Bulk insert readings
    pub fn bulk_insert(readings: &[Reading]) -> Result<usize> {
        let mut conn = get_connection()?;
        let tx = conn.transaction()?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Time went backwards")?
            .as_secs() as i64;
        
        let mut stmt = tx.prepare(
            "INSERT INTO readings (
                timestamp, sensor_id, value, state, change_type
            ) VALUES (?, ?, ?, ?, ?)"
        )?;
        
        let mut count = 0;
        
        for reading in readings {
            // Use current time if timestamp is not provided
            let timestamp = reading.timestamp.unwrap_or(now);
            
            stmt.execute(params![
                timestamp,
                reading.sensor_id,
                reading.value,
                reading.state,
                reading.change_type
            ])?;
            
            count += 1;
        }
        
        tx.commit()?;
        
        Ok(count)
    }
    
    /// Get readings based on query parameters
    pub fn get(query: &ReadingQuery) -> Result<Vec<ReadingResponse>> {
        let conn = get_connection()?;
        
        let mut sql = String::from("SELECT * FROM readings WHERE 1=1");
        let mut params = Vec::new();
        
        if let Some(sensor_id) = query.sensor_id {
            sql.push_str(" AND sensor_id = ?");
            params.push(sensor_id.to_string());
        }
        
        if let Some(start_time) = query.start_time {
            sql.push_str(" AND timestamp >= ?");
            params.push(start_time.to_string());
        }
        
        if let Some(end_time) = query.end_time {
            sql.push_str(" AND timestamp <= ?");
            params.push(end_time.to_string());
        }
        
        sql.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(limit.to_string());
        } else {
            sql.push_str(" LIMIT 1000"); // Default limit
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(offset.to_string());
        }
        
        let mut stmt = conn.prepare(&sql)?;
        let reading_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Self::from_row(row)
        })?;
        
        let mut readings = Vec::new();
        for reading in reading_iter {
            readings.push(reading?);
        }
        
        Ok(readings)
    }
    
    /// Get the current reading for a sensor
    pub fn get_current(sensor_id: i64) -> Result<ReadingResponse> {
        let conn = get_connection()?;
        
        let reading = conn.query_row(
            "SELECT * FROM readings 
             WHERE sensor_id = ? 
             ORDER BY timestamp DESC 
             LIMIT 1",
            params![sensor_id],
            |row| Self::from_row(row),
        )?;
        
        Ok(reading)
    }
    
    /// Delete readings in a time range
    pub fn delete_range(sensor_id: Option<i64>, start_time: i64, end_time: i64) -> Result<usize> {
        let conn = get_connection()?;
        
        let mut sql = String::from(
            "DELETE FROM readings WHERE timestamp >= ? AND timestamp <= ?"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(start_time),
            Box::new(end_time),
        ];
        
        if let Some(id) = sensor_id {
            sql.push_str(" AND sensor_id = ?");
            params.push(Box::new(id));
        }
        
        let count = conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
        
        Ok(count)
    }
    
    /// Convert a database row to a ReadingResponse
    fn from_row(row: &Row) -> Result<ReadingResponse, rusqlite::Error> {
        let reading_id: i64 = row.get("reading_id")?;
        let timestamp: i64 = row.get("timestamp")?;
        let sensor_id: i64 = row.get("sensor_id")?;
        let value: Option<f64> = row.get("value")?;
        let state: Option<i64> = row.get("state")?;
        let change_type: Option<String> = row.get("change_type")?;
        
        let timestamp = DateTime::from_timestamp(timestamp, 0)
            .expect("Invalid timestamp");
        
        Ok(ReadingResponse {
            reading_id,
            timestamp,
            sensor_id,
            value,
            state,
            change_type,
        })
    }
}