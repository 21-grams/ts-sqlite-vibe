use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::get_connection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingSession {
    pub session_id: Option<i64>,
    pub sensor_id: i64,
    pub start_time: Option<i64>,  // Will be set automatically if not provided
    pub end_time: Option<i64>,    // NULL if session is ongoing
    pub sample_rate: Option<i64>, // Sampling frequency in seconds
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingSessionResponse {
    pub session_id: i64,
    pub sensor_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub sample_rate: Option<i64>,
    pub notes: Option<String>,
    pub is_active: bool,
}

impl LoggingSession {
    /// Start a new logging session
    pub fn start(&self) -> Result<i64> {
        let conn = get_connection()?;
        
        // Check if there's already an active session for this sensor
        let active_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM logging_sessions 
             WHERE sensor_id = ? AND end_time IS NULL",
            params![self.sensor_id],
            |row| row.get(0),
        )?;
        
        if active_count > 0 {
            return Err(anyhow::anyhow!("Sensor already has an active logging session"));
        }
        
        // Use current time if start_time is not provided
        let start_time = self.start_time.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64
        });
        
        let result = conn.execute(
            "INSERT INTO logging_sessions (
                sensor_id, start_time, end_time, sample_rate, notes
            ) VALUES (?, ?, ?, ?, ?)",
            params![
                self.sensor_id,
                start_time,
                self.end_time,
                self.sample_rate,
                self.notes
            ],
        )?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to start logging session"));
        }
        
        let id = conn.last_insert_rowid();
        Ok(id)
    }
    
    /// End an active logging session
    pub fn end(sensor_id: i64) -> Result<()> {
        let conn = get_connection()?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Time went backwards")?
            .as_secs() as i64;
        
        let result = conn.execute(
            "UPDATE logging_sessions 
             SET end_time = ? 
             WHERE sensor_id = ? AND end_time IS NULL",
            params![now, sensor_id],
        )?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("No active logging session found for this sensor"));
        }
        
        Ok(())
    }
    
    /// Get all sessions for a sensor
    pub fn get_by_sensor(sensor_id: i64) -> Result<Vec<LoggingSessionResponse>> {
        let conn = get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM logging_sessions 
             WHERE sensor_id = ? 
             ORDER BY start_time DESC"
        )?;
        
        let session_iter = stmt.query_map(params![sensor_id], |row| {
            Self::from_row(row)
        })?;
        
        let mut sessions = Vec::new();
        for session in session_iter {
            sessions.push(session?);
        }
        
        Ok(sessions)
    }
    
    /// Get active session for a sensor (if any)
    pub fn get_active(sensor_id: i64) -> Result<Option<LoggingSessionResponse>> {
        let conn = get_connection()?;
        
        let session = conn.query_row(
            "SELECT * FROM logging_sessions 
             WHERE sensor_id = ? AND end_time IS NULL 
             LIMIT 1",
            params![sensor_id],
            |row| Self::from_row(row),
        );
        
        match session {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
    
    /// Get all active sessions
    pub fn get_all_active() -> Result<Vec<LoggingSessionResponse>> {
        let conn = get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM logging_sessions 
             WHERE end_time IS NULL 
             ORDER BY start_time DESC"
        )?;
        
        let session_iter = stmt.query_map([], |row| {
            Self::from_row(row)
        })?;
        
        let mut sessions = Vec::new();
        for session in session_iter {
            sessions.push(session?);
        }
        
        Ok(sessions)
    }
    
    /// Convert a database row to a LoggingSessionResponse
    fn from_row(row: &Row) -> Result<LoggingSessionResponse, rusqlite::Error> {
        let session_id: i64 = row.get("session_id")?;
        let sensor_id: i64 = row.get("sensor_id")?;
        let start_time: i64 = row.get("start_time")?;
        let end_time: Option<i64> = row.get("end_time")?;
        let sample_rate: Option<i64> = row.get("sample_rate")?;
        let notes: Option<String> = row.get("notes")?;
        
        let start_time_dt = DateTime::from_timestamp(start_time, 0)
            .expect("Invalid timestamp");
        
        let end_time_dt = end_time.map(|ts| {
            DateTime::from_timestamp(ts, 0).expect("Invalid timestamp")
        });
        
        Ok(LoggingSessionResponse {
            session_id,
            sensor_id,
            start_time: start_time_dt,
            end_time: end_time_dt,
            sample_rate,
            notes,
            is_active: end_time.is_none(),
        })
    }
}