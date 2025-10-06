mod telemetry;

use chrono::Utc;
use std::env;
use telemetry::Telemetry;
use rand::Rng;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    let mqtt_broker = env::var("MQTT_BROKER").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_port: u16 = env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);
    let rate: u64 = env::var("RATE")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);
    let num_devices: usize = env::var("DEVICES")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting IoT Simulator");
    info!("Broker: {}:{}, Rate: {} msg/s, Devices: {}", mqtt_broker, mqtt_port, rate, num_devices);

    // Generate client ID 
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let client_id = format!("sim-{}", rng.gen::<u32>());

    // Connect to MQTT broker
    let mut mqtt_options = MqttOptions::new(&client_id, &mqtt_broker, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));
    mqtt_options.set_clean_session(true);

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 20000);

    // Spawn eventloop handler
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    error!("MQTT eventloop error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    info!("Connected to MQTT broker, starting to publish telemetry");

    let mut rng = rand::thread_rng();
    let mut counter = 0u64;

    const BURST_SIZE: usize = 200;
    let burst_interval = Duration::from_millis((BURST_SIZE as u64 * 1000) / rate);
    
    info!("Publishing in bursts of {} messages every {:?}", BURST_SIZE, burst_interval);

    loop {
        let burst_start = std::time::Instant::now();

        for _ in 0..BURST_SIZE {
            let device_id = format!("dev-{}", counter % num_devices as u64);
            let telemetry = generate_telemetry(&mut rng, device_id);

            let topic = format!("telemetry/{}", telemetry.device_id);
            let payload = match serde_json::to_string(&telemetry) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to serialize telemetry: {}", e);
                    continue;
                }
            };

            match client.publish(&topic, QoS::AtLeastOnce, false, payload).await {
                Ok(_) => {
                    counter += 1;
                }
                Err(e) => {
                    warn!("Failed to publish: {}", e);
                }
            }
        }
        
        // Log progress periodically
        if counter % 10000 == 0 {
            info!("Published {} messages", counter);
        }

        let elapsed = burst_start.elapsed();
        if elapsed < burst_interval {
            tokio::time::sleep(burst_interval - elapsed).await;
        } else if elapsed > burst_interval * 2 {
            warn!("Burst took {:?}, target was {:?} - system may be overloaded", elapsed, burst_interval);
        }
    }
}

fn generate_telemetry(rng: &mut impl Rng, device_id: String) -> Telemetry {
    let temperature = if rng.gen_bool(0.05) {
        rng.gen_range(-50.0..100.0) // 5% outliers
    } else {
        rng.gen_range(15.0..35.0) // Normal range
    };

    let humidity = if rng.gen_bool(0.05) {
        rng.gen_range(0.0..100.0) // 5% outliers
    } else {
        rng.gen_range(30.0..80.0) // Normal range
    };

    let battery = if rng.gen_bool(0.02) {
        rng.gen_range(0.0..20.0) // 2% low battery
    } else {
        rng.gen_range(20.0..100.0) // Normal range
    };

    Telemetry {
        device_id,
        timestamp: Utc::now(),
        temperature,
        humidity,
        battery,
    }
}
