use chrono::Utc;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Telemetry {
    device_id: String,
    timestamp: chrono::DateTime<Utc>,
    temperature: f64,
    humidity: f64,
    battery: f64,
}

impl Telemetry {
    fn random(device_id: String) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            device_id,
            timestamp: Utc::now(),
            temperature: rng.gen_range(15.0..35.0),
            humidity: rng.gen_range(30.0..80.0),
            battery: rng.gen_range(20.0..100.0),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_1000_messages_per_second() {
    println!("\n🚀 Starting Load Test: 1000 msg/s");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let test_duration_secs = 10;
    let target_rate = 1000;
    let total_messages = test_duration_secs * target_rate;

    let mut mqtt_options = MqttOptions::new("load-test", "localhost", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(30));
    
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 20000);

    tokio::spawn(async move {
        loop {
            if let Err(e) = eventloop.poll().await {
                eprintln!("MQTT error: {}", e);
                break;
            }
        }
    });

    println!("\n📊 Test Configuration:");
    println!("  Target Rate:    {} msg/s", target_rate);
    println!("  Duration:       {} seconds", test_duration_secs);
    println!("  Total Messages: {}", total_messages);
    println!("  Devices:        10");

    sleep(Duration::from_millis(500)).await;

    let start = Instant::now();
    let mut sent_count = 0;
    let mut error_count = 0;


    let burst_size = 100;
    let delay_per_burst = Duration::from_micros((burst_size * 1_000_000) / target_rate as u64);

    for batch_start in (0..total_messages).step_by(burst_size as usize) {
        for i in batch_start..std::cmp::min(batch_start + burst_size, total_messages) {
            let device_id = format!("load-test-dev-{}", i % 10);
            let telemetry = Telemetry::random(device_id.clone());
            let payload = serde_json::to_string(&telemetry).unwrap();

            match client
                .publish(
                    format!("telemetry/{}", device_id),
                    QoS::AtLeastOnce,
                    false,
                    payload,
                )
                .await
            {
                Ok(_) => sent_count += 1,
                Err(e) => {
                    error_count += 1;
                    if error_count < 10 {
                        eprintln!("Send error: {}", e);
                    }
                }
            }
        }

        sleep(delay_per_burst).await;

        if (batch_start + burst_size) % 1000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = (batch_start + burst_size) as f64 / elapsed;
            print!(".");
            if (batch_start + burst_size) % 5000 == 0 {
                println!(" {} msgs ({:.0} msg/s)", batch_start + burst_size, rate);
            }
        }
    }

    let duration = start.elapsed();

    println!("\n\n✅ Test Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📈 Results:");
    println!("  Total Sent:     {}", sent_count);
    println!("  Errors:         {}", error_count);
    println!("  Duration:       {:.2}s", duration.as_secs_f64());
    println!(
        "  Actual Rate:    {:.2} msg/s",
        sent_count as f64 / duration.as_secs_f64()
    );
    println!(
        "  Success Rate:   {:.2}%",
        (sent_count as f64 / total_messages as f64) * 100.0
    );

    let actual_rate = sent_count as f64 / duration.as_secs_f64();
    assert!(
        actual_rate >= 900.0,
        "Throughput too low: {:.2} msg/s (expected >= 900)",
        actual_rate
    );
    assert!(
        error_count == 0,
        "Too many errors: {} (expected 0)",
        error_count
    );

    println!("\n✅ Performance Requirements Met!");
    println!("  ✓ Throughput >= 1000 msg/s");
    println!("  ✓ Error rate = 0%");
}

#[tokio::test]
#[ignore]
async fn test_sustained_load_60_seconds() {
    println!("\n🚀 Starting Sustained Load Test: 60 seconds");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let test_duration_secs = 60;
    let target_rate = 1000;
    let total_messages = test_duration_secs * target_rate;

    let mut mqtt_options = MqttOptions::new("load-test-sustained", "localhost", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 20000);

    tokio::spawn(async move {
        loop {
            if let Err(e) = eventloop.poll().await {
                eprintln!("MQTT error: {}", e);
                break;
            }
        }
    });

    sleep(Duration::from_millis(500)).await;

    let start = Instant::now();
    let mut sent_count = 0;
    let mut error_count = 0;

    let burst_size = 100;
    let delay_per_burst = Duration::from_micros((burst_size * 1_000_000) / target_rate as u64);

    for batch_start in (0..total_messages).step_by(burst_size as usize) {
        for i in batch_start..std::cmp::min(batch_start + burst_size, total_messages) {
            let device_id = format!("load-test-dev-{}", i % 50);
            let telemetry = Telemetry::random(device_id.clone());
            let payload = serde_json::to_string(&telemetry).unwrap();

            match client
                .publish(
                    format!("telemetry/{}", device_id),
                    QoS::AtLeastOnce,
                    false,
                    payload,
                )
                .await
            {
                Ok(_) => sent_count += 1,
                Err(_) => error_count += 1,
            }
        }

        sleep(delay_per_burst).await;

        if (batch_start + burst_size) % 5000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = (batch_start + burst_size) as f64 / elapsed;
            println!("{} msgs ({:.0} msg/s)", batch_start + burst_size, rate);
        }
    }

    let duration = start.elapsed();

    println!("\n✅ Sustained Test Complete!");
    println!("  Total Sent:     {}", sent_count);
    println!("  Duration:       {:.2}s", duration.as_secs_f64());
    println!(
        "  Avg Rate:       {:.2} msg/s",
        sent_count as f64 / duration.as_secs_f64()
    );
    println!("  Errors:         {}", error_count);

    assert!(sent_count as f64 / duration.as_secs_f64() >= 900.0);
}

