use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::models::{LoggingSession, LoggingSessionResponse};
use crate::utils::error::AppError;

/// Start a new logging session
pub async fn start_logging(
    Json(session): Json<LoggingSession>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let session_id = session.start()?;
    
    let response = json!({
        "success": true,
        "session_id": session_id
    });
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// End an active logging session
pub async fn end_logging(
    Path(sensor_id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    LoggingSession::end(sensor_id)?;
    
    let response = json!({
        "success": true,
        "sensor_id": sensor_id
    });
    
    Ok((StatusCode::OK, Json(response)))
}

/// Get all sessions for a sensor
pub async fn get_sessions_by_sensor(
    Path(sensor_id): Path<i64>,
) -> Result<Json<Vec<LoggingSessionResponse>>, AppError> {
    let sessions = LoggingSession::get_by_sensor(sensor_id)?;
    Ok(Json(sessions))
}

/// Get active session for a sensor (if any)
pub async fn get_active_session(
    Path(sensor_id): Path<i64>,
) -> Result<Json<Option<LoggingSessionResponse>>, AppError> {
    let session = LoggingSession::get_active(sensor_id)?;
    Ok(Json(session))
}

/// Get all active sessions
pub async fn get_all_active_sessions() -> Result<Json<Vec<LoggingSessionResponse>>, AppError> {
    let sessions = LoggingSession::get_all_active()?;
    Ok(Json(sessions))
}