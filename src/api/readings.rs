use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::models::{Reading, ReadingBulkInsert, ReadingBulkResponse, ReadingQuery, ReadingResponse};
use crate::utils::error::AppError;

/// Log a single sensor reading
pub async fn create_reading(
    Json(reading): Json<Reading>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let reading_id = reading.create()?;
    
    let response = json!({
        "success": true,
        "reading_id": reading_id
    });
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Bulk import readings
pub async fn bulk_import_readings(
    Json(payload): Json<ReadingBulkInsert>,
) -> Result<Json<ReadingBulkResponse>, AppError> {
    let inserted_count = Reading::bulk_insert(&payload.readings)?;
    
    let response = ReadingBulkResponse {
        inserted_count,
        success: true,
    };
    
    Ok(Json(response))
}

/// Get readings with filtering
pub async fn get_readings(
    Query(query): Query<ReadingQuery>,
) -> Result<Json<Vec<ReadingResponse>>, AppError> {
    let readings = Reading::get(&query)?;
    Ok(Json(readings))
}

/// Get current reading for a sensor
pub async fn get_current_reading(
    Path(sensor_id): Path<i64>,
) -> Result<Json<ReadingResponse>, AppError> {
    let reading = Reading::get_current(sensor_id)?;
    Ok(Json(reading))
}

/// Delete readings in a time range
pub async fn delete_readings(
    Query(query): Query<ReadingQuery>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // Require start_time and end_time for deletion
    let start_time = query.start_time
        .ok_or_else(|| AppError::BadRequest("start_time is required".to_string()))?;
    
    let end_time = query.end_time
        .ok_or_else(|| AppError::BadRequest("end_time is required".to_string()))?;
    
    let deleted_count = Reading::delete_range(query.sensor_id, start_time, end_time)?;
    
    let response = json!({
        "success": true,
        "deleted_count": deleted_count
    });
    
    Ok((StatusCode::OK, Json(response)))
}