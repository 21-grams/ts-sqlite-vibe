mod dashboard;
mod models;

use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::info;

use crate::models::*;

// Client state to store API endpoint and manage client
pub struct AppState {
    api_base_url: String,
    http_client: Client,
    active_sensors: RwLock<HashMap<i64, String>>,
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Fetch sensors from the API
    let response = state.http_client
        .get(&format!("{}/sensors", state.api_base_url))
        .send()
        .await;

    let mut sensor_list_html = String::new();
    
    if let Ok(response) = response {
        if response.status().is_success() {
            let sensors: Vec<SensorResponse> = response.json().await.unwrap_or_default();
            
            if sensors.is_empty() {
                sensor_list_html = "<p>No sensors found. Create your first sensor below.</p>".to_string();
            } else {
                sensor_list_html = "<table class=\"table\"><thead><tr><th>ID</th><th>Name</th><th>Type</th><th>Location</th><th>Actions</th></tr></thead><tbody>".to_string();
                
                for sensor in sensors {
                    let active_sensors = state.active_sensors.read().await;
                    let is_logging = active_sensors.contains_key(&sensor.sensor_id);
                    
                    let action_buttons = if is_logging {
                        format!("<a href=\"/readings/log/{}\" class=\"btn btn-primary btn-sm\">Log Reading</a> <a href=\"/sessions/end/{}\" class=\"btn btn-danger btn-sm\">Stop Logging</a>", 
                            sensor.sensor_id, sensor.sensor_id)
                    } else {
                        format!("<a href=\"/sessions/start/{}\" class=\"btn btn-success btn-sm\">Start Logging</a>", 
                            sensor.sensor_id)
                    };
                    
                    sensor_list_html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        sensor.sensor_id, 
                        sensor.sensor_name,
                        sensor.sensor_type,
                        sensor.location.unwrap_or_default(),
                        action_buttons
                    ));
                }
                
                sensor_list_html.push_str("</tbody></table>");
            }
        } else {
            sensor_list_html = format!("<p>Error fetching sensors: {}</p>", response.status());
        }
    } else {
        sensor_list_html = "<p>Error connecting to API server.</p>".to_string();
    }

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Sensor Monitoring Client</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
        </head>
        <body>
            <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
                <div class="container">
                    <a class="navbar-brand" href="/">Sensor Monitoring</a>
                    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
                        <span class="navbar-toggler-icon"></span>
                    </button>
                    <div class="collapse navbar-collapse" id="navbarNav">
                        <ul class="navbar-nav">
                            <li class="nav-item">
                                <a class="nav-link" href="/dashboard">Dashboard</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link active" href="/">Sensors</a>
                            </li>
                        </ul>
                    </div>
                </div>
            </nav>
            
            <div class="container mt-4">
                <h1>Sensor Management</h1>
                <div class="row mt-4">
                    <div class="col-md-12">
                        <div class="card">
                            <div class="card-header d-flex justify-content-between align-items-center">
                                <h2>Sensor List</h2>
                                <a href="/dashboard" class="btn btn-primary">View Dashboard</a>
                            </div>
                            <div class="card-body">
                                {sensor_list}
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="row mt-4">
                    <div class="col-md-6">
                        <div class="card">
                            <div class="card-header">
                                <h2>Add New Sensor</h2>
                            </div>
                            <div class="card-body">
                                <form action="/sensors/create" method="post">
                                    <div class="mb-3">
                                        <label for="sensor_name" class="form-label">Name</label>
                                        <input type="text" class="form-control" id="sensor_name" name="sensor_name" required>
                                    </div>
                                    <div class="mb-3">
                                        <label for="sensor_type" class="form-label">Type</label>
                                        <select class="form-control" id="sensor_type" name="sensor_type" required>
                                            <option value="temperature">Temperature</option>
                                            <option value="power">Power</option>
                                            <option value="flow">Flow</option>
                                            <option value="light">Light</option>
                                            <option value="humidity">Humidity</option>
                                        </select>
                                    </div>
                                    <div class="mb-3">
                                        <label for="location" class="form-label">Location</label>
                                        <input type="text" class="form-control" id="location" name="location">
                                    </div>
                                    <div class="mb-3">
                                        <label for="unit" class="form-label">Unit</label>
                                        <input type="text" class="form-control" id="unit" name="unit">
                                    </div>
                                    <div class="row">
                                        <div class="col-md-6 mb-3">
                                            <label for="threshold_min" class="form-label">Min Threshold</label>
                                            <input type="number" step="0.1" class="form-control" id="threshold_min" name="threshold_min">
                                        </div>
                                        <div class="col-md-6 mb-3">
                                            <label for="threshold_max" class="form-label">Max Threshold</label>
                                            <input type="number" step="0.1" class="form-control" id="threshold_max" name="threshold_max">
                                        </div>
                                    </div>
                                    <div class="mb-3">
                                        <label for="notes" class="form-label">Notes</label>
                                        <textarea class="form-control" id="notes" name="notes" rows="3"></textarea>
                                    </div>
                                    <button type="submit" class="btn btn-primary">Create Sensor</button>
                                </form>
                            </div>
                        </div>
                    </div>
                    
                    <div class="col-md-6">
                        <div class="card">
                            <div class="card-header">
                                <h2>Manual Reading</h2>
                            </div>
                            <div class="card-body">
                                <form action="/readings/create" method="post">
                                    <div class="mb-3">
                                        <label for="sensor_id" class="form-label">Sensor</label>
                                        <select class="form-control" id="sensor_id" name="sensor_id" required>
                                            <!-- Will be populated by JavaScript -->
                                        </select>
                                    </div>
                                    <div class="mb-3">
                                        <label for="value" class="form-label">Value</label>
                                        <input type="number" step="0.01" class="form-control" id="value" name="value" required>
                                    </div>
                                    <div class="mb-3">
                                        <label for="change_type" class="form-label">Change Type</label>
                                        <select class="form-control" id="change_type" name="change_type">
                                            <option value="periodic">Periodic</option>
                                            <option value="event">Event</option>
                                            <option value="manual">Manual</option>
                                        </select>
                                    </div>
                                    <button type="submit" class="btn btn-primary">Log Reading</button>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <script>
                // Populate the sensor dropdown in the readings form
                document.addEventListener('DOMContentLoaded', function() {
                    fetch('/api/sensors/list')
                        .then(response => response.json())
                        .then(sensors => {
                            const sensorSelect = document.getElementById('sensor_id');
                            sensors.forEach(sensor => {
                                const option = document.createElement('option');
                                option.value = sensor.sensor_id;
                                option.textContent = `${sensor.sensor_name} (${sensor.sensor_type})`;
                                sensorSelect.appendChild(option);
                            });
                        });
                });
            </script>
            <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js"></script>
        </body>
        </html>
        "#,
        sensor_list = sensor_list_html
    ))
}

async fn create_sensor(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SensorForm>,
) -> impl IntoResponse {
    let sensor = json!({
        "sensor_name": form.sensor_name,
        "sensor_type": form.sensor_type,
        "location": form.location,
        "unit": form.unit,
        "threshold_min": form.threshold_min.parse::<f64>().ok(),
        "threshold_max": form.threshold_max.parse::<f64>().ok(),
        "notes": form.notes
    });

    let response = state.http_client
        .post(&format!("{}/sensors", state.api_base_url))
        .json(&sensor)
        .send()
        .await;

    match response {
        Ok(_) => Redirect::to("/").into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error creating sensor</h1><a href=\"/\">Return to dashboard</a>".to_string()),
        )
            .into_response(),
    }
}

async fn start_session_form(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<i64>,
) -> impl IntoResponse {
    // Fetch sensor details
    let response = state.http_client
        .get(&format!("{}/sensors/{}", state.api_base_url, sensor_id))
        .send()
        .await;
    
    let sensor_details = match response {
        Ok(response) => {
            if response.status().is_success() {
                let sensor: SensorResponse = response.json().await.unwrap_or_else(|_| {
                    SensorResponse {
                        sensor_id,
                        sensor_name: "Unknown".to_string(),
                        sensor_type: "Unknown".to_string(),
                        location: None,
                        unit: None,
                        threshold_min: None,
                        threshold_max: None,
                        calibration_date: None,
                        notes: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                });
                format!(
                    "<p><strong>Sensor:</strong> {} ({})<br><strong>Location:</strong> {}</p>",
                    sensor.sensor_name,
                    sensor.sensor_type,
                    sensor.location.unwrap_or_default()
                )
            } else {
                "<p>Error fetching sensor details</p>".to_string()
            }
        }
        Err(_) => "<p>Error connecting to API server</p>".to_string(),
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Start Logging Session</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
        </head>
        <body>
            <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
                <div class="container">
                    <a class="navbar-brand" href="/">Sensor Monitoring</a>
                </div>
            </nav>
            
            <div class="container mt-4">
                <h1>Start Logging Session</h1>
                
                <div class="card mt-4">
                    <div class="card-header">
                        <h2>Sensor Details</h2>
                    </div>
                    <div class="card-body">
                        {sensor_details}
                    </div>
                </div>
                
                <div class="card mt-4">
                    <div class="card-header">
                        <h2>Session Configuration</h2>
                    </div>
                    <div class="card-body">
                        <form action="/sessions/create" method="post">
                            <input type="hidden" name="sensor_id" value="{sensor_id}">
                            
                            <div class="mb-3">
                                <label for="sample_rate" class="form-label">Sample Rate (seconds)</label>
                                <input type="number" class="form-control" id="sample_rate" name="sample_rate" value="60">
                            </div>
                            
                            <div class="mb-3">
                                <label for="notes" class="form-label">Notes</label>
                                <textarea class="form-control" id="notes" name="notes" rows="3"></textarea>
                            </div>
                            
                            <div class="d-flex justify-content-between">
                                <a href="/" class="btn btn-secondary">Cancel</a>
                                <button type="submit" class="btn btn-primary">Start Logging</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
            <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js"></script>
        </body>
        </html>
        "#,
        sensor_id = sensor_id,
        sensor_details = sensor_details
    ))
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SessionForm>,
) -> impl IntoResponse {
    let session = json!({
        "sensor_id": form.sensor_id,
        "sample_rate": form.sample_rate.parse::<i64>().ok(),
        "notes": form.notes
    });

    let response = state.http_client
        .post(&format!("{}/sessions", state.api_base_url))
        .json(&session)
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                // Add to active sensors
                let mut active_sensors = state.active_sensors.write().await;
                active_sensors.insert(form.sensor_id, "active".to_string());
                
                Redirect::to("/").into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Html("<h1>Error starting session</h1><a href=\"/\">Return to dashboard</a>".to_string()),
                )
                .into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error connecting to API</h1><a href=\"/\">Return to dashboard</a>".to_string()),
        )
            .into_response(),
    }
}

async fn end_session(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<i64>,
) -> impl IntoResponse {
    let response = state.http_client
        .post(&format!("{}/sessions/end/{}", state.api_base_url, sensor_id))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                // Remove from active sensors
                let mut active_sensors = state.active_sensors.write().await;
                active_sensors.remove(&sensor_id);
                
                Redirect::to("/").into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Html("<h1>Error ending session</h1><a href=\"/\">Return to dashboard</a>".to_string()),
                )
                .into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error connecting to API</h1><a href=\"/\">Return to dashboard</a>".to_string()),
        )
            .into_response(),
    }
}

async fn log_reading_form(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<i64>,
) -> impl IntoResponse {
    // Fetch sensor details
    let response = state.http_client
        .get(&format!("{}/sensors/{}", state.api_base_url, sensor_id))
        .send()
        .await;
    
    let sensor_details = match response {
        Ok(response) => {
            if response.status().is_success() {
                let sensor: SensorResponse = response.json().await.unwrap_or_else(|_| {
                    SensorResponse {
                        sensor_id,
                        sensor_name: "Unknown".to_string(),
                        sensor_type: "Unknown".to_string(),
                        location: None,
                        unit: None,
                        threshold_min: None,
                        threshold_max: None,
                        calibration_date: None,
                        notes: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                });
                
                let unit_display = sensor.unit.clone().unwrap_or_default();
                
                format!(
                    r#"<p><strong>Sensor:</strong> {} ({})<br>
                       <strong>Location:</strong> {}<br>
                       <strong>Unit:</strong> {}</p>"#,
                    sensor.sensor_name,
                    sensor.sensor_type,
                    sensor.location.unwrap_or_default(),
                    unit_display
                )
            } else {
                "<p>Error fetching sensor details</p>".to_string()
            }
        }
        Err(_) => "<p>Error connecting to API server</p>".to_string(),
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Log Sensor Reading</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
        </head>
        <body>
            <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
                <div class="container">
                    <a class="navbar-brand" href="/">Sensor Monitoring</a>
                </div>
            </nav>
            
            <div class="container mt-4">
                <h1>Log Sensor Reading</h1>
                
                <div class="card mt-4">
                    <div class="card-header">
                        <h2>Sensor Details</h2>
                    </div>
                    <div class="card-body">
                        {sensor_details}
                    </div>
                </div>
                
                <div class="card mt-4">
                    <div class="card-header">
                        <h2>New Reading</h2>
                    </div>
                    <div class="card-body">
                        <form action="/readings/create" method="post">
                            <input type="hidden" name="sensor_id" value="{sensor_id}">
                            
                            <div class="mb-3">
                                <label for="value" class="form-label">Value</label>
                                <input type="number" step="0.01" class="form-control" id="value" name="value" required>
                            </div>
                            
                            <div class="mb-3">
                                <label for="change_type" class="form-label">Change Type</label>
                                <select class="form-control" id="change_type" name="change_type">
                                    <option value="periodic">Periodic</option>
                                    <option value="event">Event</option>
                                    <option value="manual">Manual</option>
                                </select>
                            </div>
                            
                            <div class="d-flex justify-content-between">
                                <a href="/" class="btn btn-secondary">Cancel</a>
                                <button type="submit" class="btn btn-primary">Submit Reading</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
            <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js"></script>
        </body>
        </html>
        "#,
        sensor_id = sensor_id,
        sensor_details = sensor_details
    ))
}

async fn create_reading(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ReadingForm>,
) -> impl IntoResponse {
    let reading = json!({
        "sensor_id": form.sensor_id,
        "value": form.value.parse::<f64>().ok(),
        "change_type": form.change_type
    });

    let response = state.http_client
        .post(&format!("{}/readings", state.api_base_url))
        .json(&reading)
        .send()
        .await;

    match response {
        Ok(_) => Redirect::to("/").into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error logging reading</h1><a href=\"/\">Return to dashboard</a>".to_string()),
        )
            .into_response(),
    }
}

// API proxy endpoint for JavaScript to fetch sensor list
async fn api_get_sensors(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let response = state.http_client
        .get(&format!("{}/sensors", state.api_base_url))
        .send()
        .await;
    
    match response {
        Ok(response) => {
            if response.status().is_success() {
                let sensors: Vec<SensorResponse> = response.json().await.unwrap_or_default();
                axum::Json(sensors).into_response()
            } else {
                (StatusCode::BAD_REQUEST, "Error fetching sensors").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "API server error").into_response(),
    }
}

// API proxy for visualization endpoints
async fn api_proxy(
    State(state): State<Arc<AppState>>,
    Path(endpoint): Path<String>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut url = format!("{}/{}", state.api_base_url, endpoint);
    
    // Add query parameters
    if !params.is_empty() {
        let query_string: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        url = format!("{}?{}", url, query_string.join("&"));
    }
    
    let response = state.http_client
        .get(&url)
        .send()
        .await;
    
    match response {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.json::<Value>().await.unwrap_or_default();
                axum::Json(body).into_response()
            } else {
                let status = response.status();
                (status, "Error from API").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "API server error").into_response(),
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create client API endpoint - this would point to our sensor monitoring API
    let api_base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3000/api".to_string());
    
    info!("Using API endpoint: {}", api_base_url);
    
    // Create app state with HTTP client
    let app_state = Arc::new(AppState {
        api_base_url,
        http_client: Client::new(),
        active_sensors: RwLock::new(HashMap::new()),
    });
    
    // Build routes
    let app = Router::new()
        .route("/", get(index))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/sensors/create", post(create_sensor))
        .route("/sessions/start/:sensor_id", get(start_session_form))
        .route("/sessions/create", post(create_session))
        .route("/sessions/end/:sensor_id", get(end_session))
        .route("/readings/log/:sensor_id", get(log_reading_form))
        .route("/readings/create", post(create_reading))
        .route("/api/sensors/list", get(api_get_sensors))
        .route("/api/proxy/*endpoint", get(api_proxy))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);
    
    // Run the web server
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    
    let addr = SocketAddr::from((host.parse().unwrap(), port));
    info!("Client app listening on http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}