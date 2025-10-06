mod batching;
mod db;
mod errors;
mod metrics;
mod model;
mod mqtt;
mod rest;
mod validate;

use axum::{routing::get, Router};
use tokio::sync::mpsc;
use tracing::{error, info};
use std::env;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://iot:pass@localhost:5432/iotdb".to_string());
    let mqtt_broker = env::var("MQTT_BROKER").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_port: u16 = env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let batch_size: usize = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "2000".to_string())
        .parse()
        .unwrap_or(2000);
    let batch_timeout_ms: u64 = env::var("BATCH_TIMEOUT_MS")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .unwrap_or(20);
    let channel_capacity: usize = env::var("CHANNEL_CAPACITY")
        .unwrap_or_else(|_| "100000".to_string()) 
        .parse()
        .unwrap_or(100000);

    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting IoT Ingestor");
    info!("MQTT broker: {}:{}", mqtt_broker, mqtt_port);
    info!("HTTP server: {}", http_addr);
    info!("Database: {}", database_url.split('@').last().unwrap_or("***"));

    // Initialize metrics
    metrics::init_metrics();

    // Connect to database
    let pool = match db::make_pool(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Create bounded channel for telemetry data
    info!("Channel capacity: {}", channel_capacity);
    let (tx, rx) = mpsc::channel(channel_capacity);

    // Generate client ID
    let client_id = format!("ingestor-{}", uuid::Uuid::new_v4());
    let mqtt_handle = tokio::spawn(async move {
        if let Err(e) = mqtt::run_mqtt(mqtt_broker, mqtt_port, client_id, tx).await {
            error!("MQTT task failed: {}", e);
        }
    });

    // Spawn batcher task
    let batcher_pool = pool.clone();
    let batcher_handle = tokio::spawn(async move {
        batching::run_batcher(rx, batcher_pool, batch_size, batch_timeout_ms).await;
    });

    // Build HTTP app with REST API and metrics endpoint
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .merge(rest::create_router(pool));

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to bind to {}: {}", http_addr, e);
            std::process::exit(1);
        });

    info!("HTTP server listening on {}", http_addr);

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap_or_else(|e| {
            error!("HTTP server error: {}", e);
        });
    });

    tokio::select! {
        _ = mqtt_handle => {
            error!("MQTT task terminated");
        }
        _ = batcher_handle => {
            error!("Batcher task terminated");
        }
        _ = server_handle => {
            error!("HTTP server terminated");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    info!("Shutting down");
}

async fn metrics_handler() -> String {
    metrics::gather_metrics()
}
