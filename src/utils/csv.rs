use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::io::{Read, Write};

use crate::models::{Reading, Sensor};

/// Format for timestamp representation in CSV
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Export sensor readings to CSV format
pub fn export_readings_to_csv<W: Write>(
    writer: W,
    readings: &[crate::models::ReadingResponse],
    include_headers: bool,
) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(include_headers)
        .from_writer(writer);
    
    // Write headers
    if include_headers {
        wtr.write_record(&[
            "reading_id",
            "timestamp",
            "formatted_time",
            "sensor_id",
            "value",
            "state",
            "change_type",
        ])?;
    }
    
    // Write data rows
    for reading in readings {
        let timestamp = reading.timestamp.timestamp();
        let formatted_time = reading.timestamp.format(TIMESTAMP_FORMAT).to_string();
        
        wtr.write_record(&[
            reading.reading_id.to_string(),
            timestamp.to_string(),
            formatted_time,
            reading.sensor_id.to_string(),
            reading.value.map(|v| v.to_string()).unwrap_or_default(),
            reading.state.map(|s| s.to_string()).unwrap_or_default(),
            reading.change_type.clone().unwrap_or_default(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Export sensors to CSV format
pub fn export_sensors_to_csv<W: Write>(
    writer: W,
    sensors: &[crate::models::SensorResponse],
    include_headers: bool,
) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(include_headers)
        .from_writer(writer);
    
    // Write headers
    if include_headers {
        wtr.write_record(&[
            "sensor_id",
            "sensor_name",
            "sensor_type",
            "location",
            "unit",
            "threshold_min",
            "threshold_max",
            "calibration_date",
            "notes",
            "created_at",
            "updated_at",
        ])?;
    }
    
    // Write data rows
    for sensor in sensors {
        let calibration_date = sensor.calibration_date
            .map(|d| d.format(TIMESTAMP_FORMAT).to_string())
            .unwrap_or_default();
        
        let created_at = sensor.created_at.format(TIMESTAMP_FORMAT).to_string();
        let updated_at = sensor.updated_at.format(TIMESTAMP_FORMAT).to_string();
        
        wtr.write_record(&[
            sensor.sensor_id.to_string(),
            sensor.sensor_name.clone(),
            sensor.sensor_type.clone(),
            sensor.location.clone().unwrap_or_default(),
            sensor.unit.clone().unwrap_or_default(),
            sensor.threshold_min.map(|v| v.to_string()).unwrap_or_default(),
            sensor.threshold_max.map(|v| v.to_string()).unwrap_or_default(),
            calibration_date,
            sensor.notes.clone().unwrap_or_default(),
            created_at,
            updated_at,
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Import readings from CSV
pub fn import_readings_from_csv<R: Read>(reader: R) -> Result<Vec<Reading>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);
    
    let headers = rdr.headers()?.clone();
    
    let mut readings = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        
        // Get field positions (flexible mapping)
        let sensor_id_pos = headers.iter().position(|h| h.to_lowercase() == "sensor_id");
        let timestamp_pos = headers.iter().position(|h| h.to_lowercase() == "timestamp");
        let value_pos = headers.iter().position(|h| h.to_lowercase() == "value");
        let state_pos = headers.iter().position(|h| h.to_lowercase() == "state");
        let change_type_pos = headers.iter().position(|h| h.to_lowercase() == "change_type");
        
        // Required field: sensor_id
        let sensor_id = if let Some(pos) = sensor_id_pos {
            record.get(pos)
                .and_then(|s| s.parse::<i64>().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid or missing sensor_id"))?
        } else {
            return Err(anyhow::anyhow!("Missing sensor_id column"));
        };
        
        // Optional fields
        let timestamp = timestamp_pos
            .and_then(|pos| record.get(pos))
            .and_then(|s| s.parse::<i64>().ok());
        
        let value = value_pos
            .and_then(|pos| record.get(pos))
            .and_then(|s| if s.is_empty() { None } else { s.parse::<f64>().ok() });
        
        let state = state_pos
            .and_then(|pos| record.get(pos))
            .and_then(|s| if s.is_empty() { None } else { s.parse::<i64>().ok() });
        
        let change_type = change_type_pos
            .and_then(|pos| record.get(pos))
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        
        // Require either value or state
        if value.is_none() && state.is_none() {
            return Err(anyhow::anyhow!("Reading must have either value or state"));
        }
        
        let reading = Reading {
            reading_id: None,
            timestamp,
            sensor_id,
            value,
            state,
            change_type,
        };
        
        readings.push(reading);
    }
    
    Ok(readings)
}

/// Import sensors from CSV
pub fn import_sensors_from_csv<R: Read>(reader: R) -> Result<Vec<Sensor>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);
    
    let headers = rdr.headers()?.clone();
    
    let mut sensors = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        
        // Get field positions (flexible mapping)
        let name_pos = headers.iter().position(|h| h.to_lowercase().contains("name"));
        let type_pos = headers.iter().position(|h| h.to_lowercase().contains("type"));
        let location_pos = headers.iter().position(|h| h.to_lowercase() == "location");
        let unit_pos = headers.iter().position(|h| h.to_lowercase() == "unit");
        let min_pos = headers.iter().position(|h| h.to_lowercase().contains("min"));
        let max_pos = headers.iter().position(|h| h.to_lowercase().contains("max"));
        let notes_pos = headers.iter().position(|h| h.to_lowercase() == "notes");
        
        // Required fields: name and type
        let sensor_name = if let Some(pos) = name_pos {
            let name = record.get(pos).unwrap_or_default();
            if name.is_empty() {
                return Err(anyhow::anyhow!("Sensor name cannot be empty"));
            }
            name.to_string()
        } else {
            return Err(anyhow::anyhow!("Missing sensor name column"));
        };
        
        let sensor_type = if let Some(pos) = type_pos {
            let type_val = record.get(pos).unwrap_or_default();
            if type_val.is_empty() {
                return Err(anyhow::anyhow!("Sensor type cannot be empty"));
            }
            type_val.to_string()
        } else {
            return Err(anyhow::anyhow!("Missing sensor type column"));
        };
        
        // Optional fields
        let location = location_pos
            .and_then(|pos| record.get(pos))
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        
        let unit = unit_pos
            .and_then(|pos| record.get(pos))
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        
        let threshold_min = min_pos
            .and_then(|pos| record.get(pos))
            .and_then(|s| if s.is_empty() { None } else { s.parse::<f64>().ok() });
        
        let threshold_max = max_pos
            .and_then(|pos| record.get(pos))
            .and_then(|s| if s.is_empty() { None } else { s.parse::<f64>().ok() });
        
        let notes = notes_pos
            .and_then(|pos| record.get(pos))
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        
        let sensor = Sensor {
            sensor_id: None,
            sensor_name,
            sensor_type,
            location,
            unit,
            threshold_min,
            threshold_max,
            calibration_date: None,
            notes,
            created_at: None,
            updated_at: None,
        };
        
        sensors.push(sensor);
    }
    
    Ok(sensors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Cursor;
    
    #[test]
    fn test_export_readings_to_csv() -> Result<()> {
        // Create sample readings
        let readings = vec![
            crate::models::ReadingResponse {
                reading_id: 1,
                timestamp: Utc.with_ymd_and_hms(2025, 4, 11, 12, 30, 0).unwrap(),
                sensor_id: 1,
                value: Some(21.5),
                state: None,
                change_type: Some("periodic".to_string()),
            },
            crate::models::ReadingResponse {
                reading_id: 2,
                timestamp: Utc.with_ymd_and_hms(2025, 4, 11, 12, 35, 0).unwrap(),
                sensor_id: 1,
                value: Some(22.0),
                state: None,
                change_type: Some("periodic".to_string()),
            },
        ];
        
        // Create a buffer for the CSV output
        let mut buffer = Vec::new();
        
        // Export readings to CSV
        export_readings_to_csv(Cursor::new(&mut buffer), &readings, true)?;
        
        // Convert buffer to string
        let csv_output = String::from_utf8(buffer)?;
        
        // Basic checks
        assert!(csv_output.contains("reading_id,timestamp,formatted_time"));
        assert!(csv_output.contains("21.5"));
        assert!(csv_output.contains("22.0"));
        assert!(csv_output.contains("periodic"));
        
        Ok(())
    }
    
    #[test]
    fn test_import_readings_from_csv() -> Result<()> {
        // Sample CSV data
        let csv_data = r#"sensor_id,timestamp,value,change_type
1,1712921800,21.5,periodic
1,1712922100,22.0,periodic
2,1712921800,15.2,periodic
"#;
        
        // Import readings from CSV
        let readings = import_readings_from_csv(Cursor::new(csv_data))?;
        
        // Check results
        assert_eq!(readings.len(), 3);
        assert_eq!(readings[0].sensor_id, 1);
        assert_eq!(readings[0].value, Some(21.5));
        assert_eq!(readings[1].value, Some(22.0));
        assert_eq!(readings[2].sensor_id, 2);
        
        Ok(())
    }
}