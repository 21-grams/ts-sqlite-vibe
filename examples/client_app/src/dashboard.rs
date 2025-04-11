use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;
use crate::models::{SensorResponse, ReadingResponse};

#[derive(Debug, Deserialize)]
struct HealthStatus {
    timestamp: i64,
    healthy_count: i64,
    warning_count: i64,
    critical_count: i64,
    sensors_warning: Vec<SensorWarning>,
    sensors_critical: Vec<SensorWarning>,
}

#[derive(Debug, Deserialize)]
struct SensorWarning {
    sensor_id: i64,
    sensor_name: String,
    current_value: f64,
    threshold_min: Option<f64>,
    threshold_max: Option<f64>,
    status: String,
}

/// Dashboard view with visualizations
pub async fn dashboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Fetch all sensors
    let sensors_response = state
        .http_client
        .get(&format!("{}/sensors", state.api_base_url))
        .send()
        .await;

    let mut sensors = Vec::new();
    let mut temperature_sensor_ids = Vec::new();
    let mut power_sensor_ids = Vec::new();
    
    if let Ok(response) = sensors_response {
        if response.status().is_success() {
            let fetched_sensors: Vec<SensorResponse> = response.json().await.unwrap_or_default();
            
            // Group sensors by type
            for sensor in &fetched_sensors {
                match sensor.sensor_type.as_str() {
                    "temperature" => temperature_sensor_ids.push(sensor.sensor_id.to_string()),
                    "power" => power_sensor_ids.push(sensor.sensor_id.to_string()),
                    _ => {}
                }
            }
            
            sensors = fetched_sensors;
        }
    }
    
    // Fetch recent readings for each sensor
    let mut recent_readings = Vec::new();
    
    for sensor in &sensors {
        let readings_response = state
            .http_client
            .get(&format!("{}/readings/current/{}", state.api_base_url, sensor.sensor_id))
            .send()
            .await;
            
        if let Ok(response) = readings_response {
            if response.status().is_success() {
                if let Ok(reading) = response.json::<ReadingResponse>().await {
                    recent_readings.push((sensor.clone(), reading));
                }
            }
        }
    }
    
    // Build the readings table HTML
    let readings_table = if recent_readings.is_empty() {
        "<p>No recent readings found.</p>".to_string()
    } else {
        let mut table = r#"
        <table class="table table-striped">
            <thead>
                <tr>
                    <th>Sensor</th>
                    <th>Type</th>
                    <th>Location</th>
                    <th>Last Reading</th>
                    <th>Time</th>
                </tr>
            </thead>
            <tbody>
        "#.to_string();
        
        for (sensor, reading) in recent_readings {
            let reading_value = match (reading.value, reading.state) {
                (Some(val), _) => format!("{} {}", val, sensor.unit.unwrap_or_default()),
                (_, Some(1)) => "ON".to_string(),
                (_, Some(0)) => "OFF".to_string(),
                _ => "N/A".to_string(),
            };
            
            let time_str = reading.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            
            table.push_str(&format!(
                r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                "#,
                sensor.sensor_name,
                sensor.sensor_type,
                sensor.location.unwrap_or_default(),
                reading_value,
                time_str
            ));
        }
        
        table.push_str("</tbody></table>");
        table
    };
    
    // Check for active sessions
    let sessions_response = state
        .http_client
        .get(&format!("{}/sessions/active", state.api_base_url))
        .send()
        .await;
        
    let mut active_sessions_count = 0;
    
    if let Ok(response) = sessions_response {
        if response.status().is_success() {
            let active_sessions: Vec<serde_json::Value> = response.json().await.unwrap_or_default();
            active_sessions_count = active_sessions.len();
        }
    }
    
    // Build the temperature sensor IDs for chart
    let temperature_sensors_str = temperature_sensor_ids.join(",");
    let power_sensors_str = power_sensor_ids.join(",");
    
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Sensor Monitoring Dashboard</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
            <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            <style>
                .chart-container {{
                    height: 300px;
                    margin-bottom: 20px;
                }}
                .status-card {{
                    text-align: center;
                    padding: 15px;
                }}
                .status-number {{
                    font-size: 2rem;
                    font-weight: bold;
                }}
            </style>
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
                                <a class="nav-link active" href="/dashboard">Dashboard</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="/">Sensors</a>
                            </li>
                        </ul>
                    </div>
                </div>
            </nav>
            
            <div class="container mt-4">
                <h1>Monitoring Dashboard</h1>
                
                <div class="row mt-4">
                    <div class="col-md-4">
                        <div class="card status-card">
                            <div class="card-body">
                                <h5 class="card-title">Sensors</h5>
                                <p class="status-number">{}</p>
                                <p class="card-text">Total sensors configured</p>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card status-card">
                            <div class="card-body">
                                <h5 class="card-title">Active Sessions</h5>
                                <p class="status-number">{}</p>
                                <p class="card-text">Currently logging data</p>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card status-card">
                            <div class="card-body">
                                <h5 class="card-title">Sensor Status</h5>
                                <div class="chart-container">
                                    <canvas id="sensorStatusChart"></canvas>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="row mt-4">
                    <div class="col-md-6">
                        <div class="card">
                            <div class="card-header">
                                <h2>Temperature</h2>
                            </div>
                            <div class="card-body">
                                <div class="chart-container">
                                    <canvas id="temperatureChart" data-sensor-ids="{}"></canvas>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <div class="card">
                            <div class="card-header">
                                <h2>Power Consumption</h2>
                            </div>
                            <div class="card-body">
                                <div class="chart-container">
                                    <canvas id="powerConsumptionChart" data-sensor-ids="{}"></canvas>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="row mt-4">
                    <div class="col-md-12">
                        <div class="card">
                            <div class="card-header">
                                <h2>Recent Readings</h2>
                            </div>
                            <div class="card-body">
                                {}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <script src="/static/js/charts.js"></script>
            <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js"></script>
        </body>
        </html>
        "#,
        sensors.len(),
        active_sessions_count,
        temperature_sensors_str,
        power_sensors_str,
        readings_table
    ))
}