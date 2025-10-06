use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub temperature: f64,
    pub humidity: f64,
    pub battery: f64,
}

