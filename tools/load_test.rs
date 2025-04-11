use rand::Rng;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio::task;

const API_URL: &str = "http://localhost:3000/api";
const NUM_SENSORS: usize = 10;
const NUM_READINGS_PER_SENSOR: usize = 1000;
const BATCH_SIZE: usize = 100;
const CONCURRENT_REQUESTS: usize = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting load test for time-series database");
    println!("===========================================");
    
    let client = Client::new();
    
    // 1. Create test sensors
    println!("\nStep 1: Creating {} test sensors...", NUM_SENSORS);
    let mut sensor_ids = Vec::with_capacity(NUM_SENSORS);
    
    for i in 0..NUM_SENSORS {
        let sensor_type = match i % 3 {
            0 => "temperature",
            1 => "power",
            _ => "flow",
        };
        
        let unit = match sensor_type {
            "temperature" => "C",
            "power" => "kW",
            "flow" => "L/min",
            _ => "",
        };
        
        let sensor_data = json!({
            "sensor_name": format!("Test Sensor {}", i + 1),
            "sensor_type": sensor_type,
            "location": format!("Test Location {}", i + 1),
            "unit": unit,
            "threshold_min": 10.0,
            "threshold_max": 50.0,
            "notes": "Test sensor for load testing"
        });
        
        let response = client
            .post(&format!("{}/sensors", API_URL))
            .json(&sensor_data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: Value = response.json().await?;
            let sensor_id = data["sensor_id"].as_i64().unwrap();
            sensor_ids.push(sensor_id);
            println!("  Created sensor ID: {}", sensor_id);
        } else {
            println!("  Error creating sensor: {}", response.status());
        }
    }
    
    // 2. Generate test readings
    println!("\nStep 2: Generating test data...");
    let total_readings = NUM_SENSORS * NUM_READINGS_PER_SENSOR;
    println!("  Total readings to generate: {}", total_readings);
    
    let mut rng = rand::thread_rng();
    let start_time = Instant::now();
    
    // Create multiple worker tasks
    let mut handles = Vec::new();
    let chunk_size = NUM_SENSORS / CONCURRENT_REQUESTS;
    
    for chunk_idx in 0..CONCURRENT_REQUESTS {
        let start_idx = chunk_idx * chunk_size;
        let end_idx = if chunk_idx == CONCURRENT_REQUESTS - 1 {
            NUM_SENSORS
        } else {
            start_idx + chunk_size
        };
        
        let sensor_chunk = sensor_ids[start_idx..end_idx].to_vec();
        let client_clone = client.clone();
        
        handles.push(task::spawn(async move {
            let mut readings_inserted = 0;
            
            for &sensor_id in &sensor_chunk {
                for batch in 0..(NUM_READINGS_PER_SENSOR / BATCH_SIZE) {
                    let mut readings = Vec::with_capacity(BATCH_SIZE);
                    let base_timestamp = chrono::Utc::now().timestamp() - 86400; // Start 24 hours ago
                    
                    for i in 0..BATCH_SIZE {
                        let timestamp = base_timestamp + (batch * BATCH_SIZE + i) as i64 * 60; // One reading per minute
                        let value = match sensor_id % 3 {
                            0 => 20.0 + rng.gen_range(-5.0..5.0), // Temperature around 20°C
                            1 => 10.0 + rng.gen_range(0.0..15.0), // Power around 10kW
                            _ => 25.0 + rng.gen_range(-10.0..10.0), // Flow around 25L/min
                        };
                        
                        readings.push(json!({
                            "timestamp": timestamp,
                            "sensor_id": sensor_id,
                            "value": value,
                            "change_type": "periodic"
                        }));
                    }
                    
                    let bulk_data = json!({
                        "readings": readings
                    });
                    
                    let response = client_clone
                        .post(&format!("{}/readings/bulk", API_URL))
                        .json(&bulk_data)
                        .send()
                        .await
                        .expect("Failed to send request");
                    
                    if response.status().is_success() {
                        readings_inserted += BATCH_SIZE;
                    }
                    
                    // Small delay to avoid overwhelming the server
                    sleep(Duration::from_millis(10)).await;
                }
            }
            
            readings_inserted
        }));
    }
    
    // Collect results from all workers
    let mut total_inserted = 0;
    for handle in handles {
        total_inserted += handle.await?;
    }
    
    let elapsed = start_time.elapsed();
    let throughput = total_inserted as f64 / elapsed.as_secs_f64();
    
    println!("  Inserted {} readings in {:.2} seconds", total_inserted, elapsed.as_secs_f64());
    println!("  Throughput: {:.2} readings/second", throughput);
    
    // 3. Query performance test
    println!("\nStep 3: Testing query performance...");
    
    // Test 1: Get all readings for a single sensor
    let test_sensor_id = sensor_ids[0];
    let start_time_query = Instant::now();
    
    let response = client
        .get(&format!("{}/readings", API_URL))
        .query(&[
            ("sensor_id", test_sensor_id.to_string()),
            ("limit", "1000".to_string()),
        ])
        .send()
        .await?;
    
    let query_elapsed = start_time_query.elapsed();
    
    if response.status().is_success() {
        let data: Value = response.json().await?;
        let readings_count = data["readings"].as_array().map_or(0, |arr| arr.len());
        println!("  Query 1: Get all readings for sensor {} - {:.2} ms ({} readings)", 
                 test_sensor_id, query_elapsed.as_millis(), readings_count);
    }
    
    // Test 2: Get readings in a time range
    let now = chrono::Utc::now().timestamp();
    let one_hour_ago = now - 3600;
    
    let start_time_query = Instant::now();
    
    let response = client
        .get(&format!("{}/readings", API_URL))
        .query(&[
            ("start_time", one_hour_ago.to_string()),
            ("end_time", now.to_string()),
            ("limit", "1000".to_string()),
        ])
        .send()
        .await?;
    
    let query_elapsed = start_time_query.elapsed();
    
    if response.status().is_success() {
        let data: Value = response.json().await?;
        let readings_count = data["readings"].as_array().map_or(0, |arr| arr.len());
        println!("  Query 2: Get readings in time range - {:.2} ms ({} readings)", 
                 query_elapsed.as_millis(), readings_count);
    }
    
    // Test 3: Get current readings for all sensors (dashboard query)
    let start_time_query = Instant::now();
    
    let response = client
        .get(&format!("{}/status/current", API_URL))
        .send()
        .await?;
    
    let query_elapsed = start_time_query.elapsed();
    
    if response.status().is_success() {
        println!("  Query 3: Get current status - {:.2} ms", query_elapsed.as_millis());
    }
    
    // Test 4: Time-series visualization data
    let start_time_query = Instant::now();
    
    let response = client
        .get(&format!("{}/visualizations/time-series", API_URL))
        .query(&[
            ("sensor_ids", format!("{},{}", sensor_ids[0], sensor_ids[1])),
            ("start_time", (now - 86400).to_string()), // Last 24 hours
            ("end_time", now.to_string()),
            ("interval", "hour".to_string()),
        ])
        .send()
        .await?;
    
    let query_elapsed = start_time_query.elapsed();
    
    if response.status().is_success() {
        println!("  Query 4: Time-series visualization - {:.2} ms", query_elapsed.as_millis());
    }
    
    // 4. Cleanup (optional)
    if std::env::var("KEEP_TEST_DATA").is_err() {
        println!("\nStep 4: Cleaning up test data...");
        
        // Delete test sensors (this will cascade to readings)
        for sensor_id in sensor_ids {
            let response = client
                .delete(&format!("{}/sensors/{}", API_URL, sensor_id))
                .send()
                .await?;
            
            if response.status().is_success() {
                println!("  Deleted sensor ID: {}", sensor_id);
            }
        }
    } else {
        println!("\nSkipping cleanup as KEEP_TEST_DATA is set");
    }
    
    println!("\nLoad test completed successfully!");
    
    Ok(())
}