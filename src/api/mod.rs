pub mod sensors;
pub mod readings;
pub mod sessions;
pub mod system;

use axum::{
    routing::{get, post, put, delete},
    Router,
};

pub fn create_router() -> Router {
    Router::new()
        // Sensor routes
        .route("/api/sensors", post(sensors::create_sensor))
        .route("/api/sensors", get(sensors::get_all_sensors))
        .route("/api/sensors/:id", get(sensors::get_sensor_by_id))
        .route("/api/sensors/:id", put(sensors::update_sensor))
        .route("/api/sensors/:id", delete(sensors::delete_sensor))
        
        // Reading routes
        .route("/api/readings", post(readings::create_reading))
        .route("/api/readings/bulk", post(readings::bulk_import_readings))
        .route("/api/readings", get(readings::get_readings))
        .route("/api/readings/current/:sensor_id", get(readings::get_current_reading))
        .route("/api/readings", delete(readings::delete_readings))
        
        // Logging session routes
        .route("/api/sessions", post(sessions::start_logging))
        .route("/api/sessions/end/:sensor_id", post(sessions::end_logging))
        .route("/api/sessions/sensor/:sensor_id", get(sessions::get_sessions_by_sensor))
        .route("/api/sessions/active/:sensor_id", get(sessions::get_active_session))
        .route("/api/sessions/active", get(sessions::get_all_active_sessions))
        
        // System management routes
        .route("/api/system/health", get(system::get_database_health))
        .route("/api/system/maintenance", post(system::run_maintenance))
        .route("/api/system/export", get(system::export_data))
}