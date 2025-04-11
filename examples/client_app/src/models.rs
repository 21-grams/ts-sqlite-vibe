use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Models for sensor data
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Form model for sensor creation
#[derive(Debug, Deserialize)]
pub struct SensorForm {
    pub sensor_name: String,
    pub sensor_type: String,
    pub location: String,
    pub unit: String,
    pub threshold_min: String,
    pub threshold_max: String,
    pub notes: String,
}

/// Model for sensor readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingResponse {
    pub reading_id: i64,
    pub timestamp: DateTime<Utc>,
    pub sensor_id: i64,
    pub value: Option<f64>,
    pub state: Option<i64>,
    pub change_type: Option<String>,
}

/// Form model for logging a reading
#[derive(Debug, Deserialize)]
pub struct ReadingForm {
    pub sensor_id: i64,
    pub value: String,
    pub change_type: String,
}

/// Model for logging sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSessionResponse {
    pub session_id: i64,
    pub sensor_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub sample_rate: Option<i64>,
    pub notes: Option<String>,
    pub is_active: bool,
}

/// Form model for starting a logging session
#[derive(Debug, Deserialize)]
pub struct SessionForm {
    pub sensor_id: i64,
    pub sample_rate: String,
    pub notes: String,
}

/// Model for API response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub sensor_id: Option<i64>,
    pub session_id: Option<i64>,
    pub reading_id: Option<i64>,
}

/// Model for time-series data
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesDataset {
    pub sensor_id: i64,
    pub sensor_name: String,
    pub unit: String,
    pub data: Vec<f64>,
    pub moving_average: Option<Vec<f64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub labels: Vec<String>,
    pub datasets: Vec<TimeSeriesDataset>,
}