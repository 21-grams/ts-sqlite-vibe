use std::path::Path;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod models;
mod api;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "sensor_monitoring_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Get database path from env var or use default
    let db_path = std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "sensor_data.db".to_string());
    
    // Initialize the database
    let path = Path::new(&db_path);
    db::init_pool(path)?;
    
    tracing::info!("Initialized database at {}", db_path);
    
    // Create API router
    let app = api::create_router()
        .layer(TraceLayer::new_for_http());
    
    // Get port from env var or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    
    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}