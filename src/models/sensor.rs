use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::get_connection;

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crate::{
        models::Sensor,
        utils::test_utils::{setup_test_db, create_test_sensor},
    };

    #[test]
    fn test_create_sensor() -> Result<()> {
        let _pool = setup_test_db()?;
        
        let sensor = Sensor {
            sensor_id: None,
            sensor_name: "Test Temperature Sensor".to_string(),
            sensor_type: "temperature".to_string(),
            location: Some("Building A, Room 101".to_string()),
            unit: Some("C".to_string()),
            threshold_min: Some(18.0),
            threshold_max: Some(25.0),
            calibration_date: None,
            notes: Some("Test sensor".to_string()),
            created_at: None,
            updated_at: None,
        };
        
        let id = sensor.create()?;
        assert!(id > 0, "Sensor ID should be positive");
        
        let retrieved = Sensor::get_by_id(id)?;
        assert_eq!(retrieved.sensor_name, "Test Temperature Sensor");
        assert_eq!(retrieved.sensor_type, "temperature");
        assert_eq!(retrieved.location, Some("Building A, Room 101".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_update_sensor() -> Result<()> {
        let pool = setup_test_db()?;
        let conn = pool.get()?;
        
        let sensor_id = create_test_sensor(&conn)?;
        
        let sensor = Sensor {
            sensor_id: None,
            sensor_name: "Updated Sensor".to_string(),
            sensor_type: "humidity".to_string(),
            location: Some("New Location".to_string()),
            unit: Some("%".to_string()),
            threshold_min: Some(30.0),
            threshold_max: Some(70.0),
            calibration_date: None,
            notes: Some("Updated notes".to_string()),
            created_at: None,
            updated_at: None,
        };
        
        sensor.update(sensor_id)?;
        
        let retrieved = Sensor::get_by_id(sensor_id)?;
        assert_eq!(retrieved.sensor_name, "Updated Sensor");
        assert_eq!(retrieved.sensor_type, "humidity");
        assert_eq!(retrieved.unit, Some("%".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_delete_sensor() -> Result<()> {
        let pool = setup_test_db()?;
        let conn = pool.get()?;
        
        let sensor_id = create_test_sensor(&conn)?;
        
        Sensor::delete(sensor_id)?;
        
        let result = Sensor::get_by_id(sensor_id);
        assert!(result.is_err(), "Sensor should be deleted");
        
        Ok(())
    }
    
    #[test]
    fn test_get_all_sensors() -> Result<()> {
        let pool = setup_test_db()?;
        let conn = pool.get()?;
        
        // Create two sensors
        create_test_sensor(&conn)?;
        
        let sensor2 = Sensor {
            sensor_id: None,
            sensor_name: "Test Flow Sensor".to_string(),
            sensor_type: "flow".to_string(),
            location: Some("Building B".to_string()),
            unit: Some("L/min".to_string()),
            threshold_min: Some(5.0),
            threshold_max: Some(50.0),
            calibration_date: None,
            notes: Some("Test flow sensor".to_string()),
            created_at: None,
            updated_at: None,
        };
        
        sensor2.create()?;
        
        // Test with no filters
        let query = crate::models::SensorQuery {
            sensor_type: None,
            location: None,
        };
        
        let sensors = Sensor::get_all(&query)?;
        assert_eq!(sensors.len(), 2, "Should retrieve 2 sensors");
        
        // Test with type filter
        let query = crate::models::SensorQuery {
            sensor_type: Some("flow".to_string()),
            location: None,
        };
        
        let sensors = Sensor::get_all(&query)?;
        assert_eq!(sensors.len(), 1, "Should retrieve 1 sensor");
        assert_eq!(sensors[0].sensor_type, "flow");
        
        // Test with location filter
        let query = crate::models::SensorQuery {
            sensor_type: None,
            location: Some("Building B".to_string()),
        };
        
        let sensors = Sensor::get_all(&query)?;
        assert_eq!(sensors.len(), 1, "Should retrieve 1 sensor");
        assert_eq!(sensors[0].location, Some("Building B".to_string()));
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sensor {
    pub sensor_id: Option<i64>,
    pub sensor_name: String,
    pub sensor_type: String,
    pub location: Option<String>,
    pub unit: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub calibration_date: Option<i64>,
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorResponse {
    pub sensor_id: i64,
    pub sensor_name: String,
    pub sensor_type: String,
    pub location: Option<String>,
    pub unit: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub calibration_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SensorQuery {
    pub sensor_type: Option<String>,
    pub location: Option<String>,
}

impl Sensor {
    /// Create a new sensor
    pub fn create(&self) -> Result<i64> {
        let conn = get_connection()?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Time went backwards")?
            .as_secs() as i64;
        
        let result = conn.execute(
            "INSERT INTO sensors (
                sensor_name, sensor_type, location, unit, 
                threshold_min, threshold_max, calibration_date, notes,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                self.sensor_name, 
                self.sensor_type, 
                self.location, 
                self.unit,
                self.threshold_min, 
                self.threshold_max, 
                self.calibration_date, 
                self.notes,
                now, 
                now
            ],
        )?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to create sensor"));
        }
        
        let id = conn.last_insert_rowid();
        Ok(id)
    }
    
    /// Get a sensor by ID
    pub fn get_by_id(id: i64) -> Result<SensorResponse> {
        let conn = get_connection()?;
        
        let sensor = conn.query_row(
            "SELECT * FROM sensors WHERE sensor_id = ?",
            params![id],
            |row| Self::from_row(row),
        )?;
        
        Ok(sensor)
    }
    
    /// Get all sensors with optional filtering
    pub fn get_all(query: &SensorQuery) -> Result<Vec<SensorResponse>> {
        let conn = get_connection()?;
        
        let mut sql = String::from("SELECT * FROM sensors WHERE 1=1");
        let mut params = Vec::new();
        
        if let Some(ref sensor_type) = query.sensor_type {
            sql.push_str(" AND sensor_type = ?");
            params.push(sensor_type.to_string());
        }
        
        if let Some(ref location) = query.location {
            sql.push_str(" AND location = ?");
            params.push(location.to_string());
        }
        
        let mut stmt = conn.prepare(&sql)?;
        let sensor_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Self::from_row(row)
        })?;
        
        let mut sensors = Vec::new();
        for sensor in sensor_iter {
            sensors.push(sensor?);
        }
        
        Ok(sensors)
    }
    
    /// Update a sensor
    pub fn update(&self, id: i64) -> Result<()> {
        let conn = get_connection()?;
        
        let result = conn.execute(
            "UPDATE sensors SET 
                sensor_name = COALESCE(?, sensor_name),
                sensor_type = COALESCE(?, sensor_type),
                location = ?,
                unit = ?,
                threshold_min = ?,
                threshold_max = ?,
                calibration_date = ?,
                notes = ?
             WHERE sensor_id = ?",
            params![
                self.sensor_name, 
                self.sensor_type, 
                self.location, 
                self.unit,
                self.threshold_min, 
                self.threshold_max, 
                self.calibration_date, 
                self.notes,
                id
            ],
        )?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("Sensor not found or no changes made"));
        }
        
        Ok(())
    }
    
    /// Delete a sensor
    pub fn delete(id: i64) -> Result<()> {
        let conn = get_connection()?;
        
        let result = conn.execute("DELETE FROM sensors WHERE sensor_id = ?", params![id])?;
        
        if result == 0 {
            return Err(anyhow::anyhow!("Sensor not found"));
        }
        
        Ok(())
    }
    
    /// Convert a database row to a SensorResponse
    fn from_row(row: &Row) -> Result<SensorResponse, rusqlite::Error> {
        let sensor_id: i64 = row.get("sensor_id")?;
        let sensor_name: String = row.get("sensor_name")?;
        let sensor_type: String = row.get("sensor_type")?;
        let location: Option<String> = row.get("location")?;
        let unit: Option<String> = row.get("unit")?;
        let threshold_min: Option<f64> = row.get("threshold_min")?;
        let threshold_max: Option<f64> = row.get("threshold_max")?;
        let calibration_date: Option<i64> = row.get("calibration_date")?;
        let notes: Option<String> = row.get("notes")?;
        let created_at: i64 = row.get("created_at")?;
        let updated_at: i64 = row.get("updated_at")?;
        
        let calibration_date = calibration_date.map(|ts| {
            DateTime::from_timestamp(ts, 0).expect("Invalid timestamp")
        });
        
        let created_at = DateTime::from_timestamp(created_at, 0)
            .expect("Invalid timestamp");
        
        let updated_at = DateTime::from_timestamp(updated_at, 0)
            .expect("Invalid timestamp");
        
        Ok(SensorResponse {
            sensor_id,
            sensor_name,
            sensor_type,
            location,
            unit,
            threshold_min,
            threshold_max,
            calibration_date,
            notes,
            created_at,
            updated_at,
        })
    }
}