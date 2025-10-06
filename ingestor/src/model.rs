use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// IoT device telemetry data
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Telemetry {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub temperature: f64,
    pub humidity: f64,
    pub battery: f64,
}

/// REST API response wrapper
#[derive(Debug, Serialize)]
pub struct TelemetryResponse {
    pub data: Vec<Telemetry>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}
