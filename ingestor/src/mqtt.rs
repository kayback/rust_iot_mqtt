use crate::errors::{Error, Result};
use crate::metrics::{CHANNEL_FULL_TOTAL, INVALID_MESSAGES_TOTAL, MESSAGES_TOTAL, VALID_MESSAGES_TOTAL};
use crate::model::Telemetry;
use crate::validate::validate;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
const MAX_BACKOFF_MS: u64 = 2000;

pub async fn run_mqtt(
    broker: String,
    port: u16,
    client_id: String,
    tx: mpsc::Sender<Telemetry>,
) -> Result<()> {
    info!("Connecting to MQTT broker at {}:{}", broker, port);

    let mut mqtt_options = MqttOptions::new(client_id, broker, port);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(30));
    mqtt_options.set_clean_session(false);

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10000);

    // Subscribe to telemetry topic with QoS 1
    let topic = "telemetry/#";
    client
        .subscribe(topic, QoS::AtLeastOnce)
        .await
        .map_err(Error::Mqtt)?;

    info!("Subscribed to {} with QoS 1", topic);

    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    MESSAGES_TOTAL.inc();

                    debug!(
                        "Received message on topic {}, size: {} bytes",
                        publish.topic,
                        publish.payload.len()
                    );

                    // Process message with retry logic
                    if let Err(e) = process_message_with_retry(&publish.payload, &tx).await {
                        error!("Failed to process message after retries: {}", e);
                        INVALID_MESSAGES_TOTAL.inc();
                    }
                }
            }
            Err(e) => {
                error!("MQTT error: {}", e);
                // rumqttc automatically reconnects, so we just log and continue
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// Process a message with exponential backoff retry
async fn process_message_with_retry(
    payload: &[u8],
    tx: &mpsc::Sender<Telemetry>,
) -> Result<()> {
    let mut attempt = 0;
    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        attempt += 1;

        match process_message(payload, tx).await {
            Ok(()) => {
                if attempt > 1 {
                    info!("Message processed successfully on attempt {}", attempt);
                }
                return Ok(());
            }
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    return Err(e);
                }

                // Check if error is retryable
                if !is_retryable_error(&e) {
                    warn!("Non-retryable error: {}", e);
                    return Err(e);
                }

                warn!(
                    "Message processing failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempt, MAX_RETRIES, e, backoff_ms
                );

                // Wait with exponential backoff
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;

                // Increase backoff for next attempt
                backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
            }
        }
    }
}

/// Process a single message
async fn process_message(payload: &[u8], tx: &mpsc::Sender<Telemetry>) -> Result<()> {
    // Parse JSON
    let telemetry = serde_json::from_slice::<Telemetry>(payload)
        .map_err(|e| Error::Validation(format!("JSON parse error: {}", e)))?;

    // Validate
    validate(&telemetry)?;

    match tx.try_send(telemetry) {
        Ok(()) => {
            VALID_MESSAGES_TOTAL.inc();
            Ok(())
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(telemetry)) => {
            CHANNEL_FULL_TOTAL.inc();
            debug!("Channel full, using blocking send");
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            tx.send(telemetry)
                .await
                .map_err(|_| Error::ChannelSend)?;
            VALID_MESSAGES_TOTAL.inc();
            Ok(())
        }
        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
            error!("Channel closed, cannot send telemetry");
            Err(Error::ChannelSend)
        }
    }
}

/// Determine if an error is retryable
fn is_retryable_error(error: &Error) -> bool {
    match error {
        // Retryable errors
        Error::ChannelSend => true, // Channel might be temporarily full
        Error::Database(_) => true, // Database might be temporarily unavailable

        // Non-retryable errors
        Error::Validation(_) => false, // Bad data won't become valid with retry
        Error::Mqtt(_) => false,       // MQTT errors handled at connection level
        Error::Json(_) => false,       // JSON parse errors won't be fixed by retry
        Error::Io(_) => false,
        Error::Migration(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_retryable_errors() {
        assert!(is_retryable_error(&Error::ChannelSend));
        assert!(!is_retryable_error(&Error::Validation(
            "test".to_string()
        )));
    }

    #[test]
    fn test_process_message_valid() {
        tokio_test::block_on(async {
            let (tx, mut rx) = mpsc::channel(10);

            let telemetry = Telemetry {
                device_id: "test-dev".to_string(),
                timestamp: Utc::now(),
                temperature: 25.0,
                humidity: 60.0,
                battery: 80.0,
            };

            let payload = serde_json::to_vec(&telemetry).unwrap();

            assert!(process_message(&payload, &tx).await.is_ok());

            let received = rx.recv().await.unwrap();
            assert_eq!(received.device_id, "test-dev");
        });
    }

    #[test]
    fn test_process_message_invalid_json() {
        tokio_test::block_on(async {
            let (tx, _rx) = mpsc::channel(10);
            let payload = b"invalid json";

            assert!(process_message(payload, &tx).await.is_err());
        });
    }

    #[test]
    fn test_process_message_invalid_temperature() {
        tokio_test::block_on(async {
            let (tx, _rx) = mpsc::channel(10);

            let telemetry = Telemetry {
                device_id: "test-dev".to_string(),
                timestamp: Utc::now(),
                temperature: 999.0, // Out of range
                humidity: 60.0,
                battery: 80.0,
            };

            let payload = serde_json::to_vec(&telemetry).unwrap();

            assert!(process_message(&payload, &tx).await.is_err());
        });
    }
}
