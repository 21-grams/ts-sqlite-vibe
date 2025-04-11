use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(err) => {
                if err.to_string().contains("UNIQUE constraint failed") {
                    (StatusCode::CONFLICT, format!("Resource already exists: {}", err))
                } else if err.to_string().contains("FOREIGN KEY constraint failed") {
                    (StatusCode::BAD_REQUEST, format!("Referenced resource does not exist: {}", err))
                } else if err == rusqlite::Error::QueryReturnedNoRows {
                    (StatusCode::NOT_FOUND, "Resource not found".to_string())
                } else {
                    tracing::error!("Database error: {:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                }
            },
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };
        
        let body = Json(json!({
            "error": message
        }));
        
        (status, body).into_response()
    }
}