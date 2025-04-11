use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::models::{Sensor, SensorQuery, SensorResponse};
use crate::utils::error::AppError;

/// Create a new sensor
pub async fn create_sensor(
    Json(sensor): Json<Sensor>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let sensor_id = sensor.create()?;
    
    let response = json!({
        "success": true,
        "sensor_id": sensor_id
    });
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get all sensors with optional filtering
pub async fn get_all_sensors(
    Query(query): Query<SensorQuery>,
) -> Result<Json<Vec<SensorResponse>>, AppError> {
    let sensors = Sensor::get_all(&query)?;
    Ok(Json(sensors))
}

/// Get a sensor by ID
pub async fn get_sensor_by_id(
    Path(id): Path<i64>,
) -> Result<Json<SensorResponse>, AppError> {
    let sensor = Sensor::get_by_id(id)?;
    Ok(Json(sensor))
}

/// Update a sensor
pub async fn update_sensor(
    Path(id): Path<i64>,
    Json(sensor): Json<Sensor>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    sensor.update(id)?;
    
    let response = json!({
        "success": true,
        "sensor_id": id
    });
    
    Ok((StatusCode::OK, Json(response)))
}

/// Delete a sensor
pub async fn delete_sensor(
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    Sensor::delete(id)?;
    
    let response = json!({
        "success": true,
        "sensor_id": id
    });
    
    Ok((StatusCode::OK, Json(response)))
}