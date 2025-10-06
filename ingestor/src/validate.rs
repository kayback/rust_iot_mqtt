use crate::errors::{Error, Result};
use crate::model::Telemetry;

const TEMP_MIN: f64 = -50.0;
const TEMP_MAX: f64 = 100.0;
const HUMIDITY_MIN: f64 = 0.0;
const HUMIDITY_MAX: f64 = 100.0;
const BATTERY_MIN: f64 = 0.0;
const BATTERY_MAX: f64 = 100.0;

/// Validates telemetry data
pub fn validate(telemetry: &Telemetry) -> Result<()> {
    // Validate temperature
    if telemetry.temperature < TEMP_MIN || telemetry.temperature > TEMP_MAX {
        return Err(Error::Validation(format!(
            "Temperature {} out of range [{}, {}]",
            telemetry.temperature, TEMP_MIN, TEMP_MAX
        )));
    }

    // Validate humidity
    if telemetry.humidity < HUMIDITY_MIN || telemetry.humidity > HUMIDITY_MAX {
        return Err(Error::Validation(format!(
            "Humidity {} out of range [{}, {}]",
            telemetry.humidity, HUMIDITY_MIN, HUMIDITY_MAX
        )));
    }

    // Validate battery
    if telemetry.battery < BATTERY_MIN || telemetry.battery > BATTERY_MAX {
        return Err(Error::Validation(format!(
            "Battery {} out of range [{}, {}]",
            telemetry.battery, BATTERY_MIN, BATTERY_MAX
        )));
    }

    // Validate device_id
    if telemetry.device_id.is_empty() {
        return Err(Error::Validation("Device ID cannot be empty".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_valid_telemetry() {
        let telemetry = Telemetry {
            device_id: "dev-1".to_string(),
            timestamp: Utc::now(),
            temperature: 25.0,
            humidity: 60.0,
            battery: 80.0,
        };

        assert!(validate(&telemetry).is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let telemetry = Telemetry {
            device_id: "dev-1".to_string(),
            timestamp: Utc::now(),
            temperature: 150.0, // Out of range
            humidity: 60.0,
            battery: 80.0,
        };

        assert!(validate(&telemetry).is_err());
    }

    #[test]
    fn test_invalid_humidity() {
        let telemetry = Telemetry {
            device_id: "dev-1".to_string(),
            timestamp: Utc::now(),
            temperature: 25.0,
            humidity: 150.0, // Out of range
            battery: 80.0,
        };

        assert!(validate(&telemetry).is_err());
    }

    #[test]
    fn test_invalid_battery() {
        let telemetry = Telemetry {
            device_id: "dev-1".to_string(),
            timestamp: Utc::now(),
            temperature: 25.0,
            humidity: 60.0,
            battery: 150.0, // Out of range
        };

        assert!(validate(&telemetry).is_err());
    }

    #[test]
    fn test_empty_device_id() {
        let telemetry = Telemetry {
            device_id: "".to_string(),
            timestamp: Utc::now(),
            temperature: 25.0,
            humidity: 60.0,
            battery: 80.0,
        };

        assert!(validate(&telemetry).is_err());
    }
}
