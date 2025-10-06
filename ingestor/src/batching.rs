use crate::db::insert_batch;
use crate::metrics::{BATCH_SIZE, INGEST_LATENCY_SECONDS};
use crate::model::Telemetry;
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, Instant};
use tracing::{debug, error, info};

pub async fn run_batcher(
    mut rx: mpsc::Receiver<Telemetry>,
    pool: PgPool,
    max_batch: usize,
    max_wait_ms: u64,
) {
    info!(
        "Starting batcher with max_batch={}, max_wait_ms={}",
        max_batch, max_wait_ms
    );

    let mut buffer: Vec<Telemetry> = Vec::with_capacity(max_batch);
    let mut ticker = interval(Duration::from_millis(max_wait_ms));

    loop {
        tokio::select! {
            // Receive telemetry data
            telemetry = rx.recv() => {
                match telemetry {
                    Some(t) => {
                        buffer.push(t);

                        // Flush if buffer is full
                        if buffer.len() >= max_batch {
                            flush_batch(&pool, &mut buffer).await;
                        }
                    }
                    None => {
                        // Channel closed, flush remaining and exit
                        info!("Channel closed, flushing remaining batch");
                        flush_batch(&pool, &mut buffer).await;
                        break;
                    }
                }
            }

            // Periodic flush timer
            _ = ticker.tick() => {
                if !buffer.is_empty() {
                    flush_batch(&pool, &mut buffer).await;
                }
            }
        }
    }

    info!("Batcher stopped");
}

async fn flush_batch(pool: &PgPool, buffer: &mut Vec<Telemetry>) {
    let batch_len = buffer.len();
    if batch_len == 0 {
        return;
    }

    debug!("Flushing batch of {} records", batch_len);
    BATCH_SIZE.set(batch_len as f64);

    let start = Instant::now();

    // Retry logic: 3 attempts with exponential backoff
    const MAX_RETRIES: u32 = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match insert_batch(pool, buffer).await {
            Ok(()) => {
                let elapsed = start.elapsed().as_secs_f64();
                INGEST_LATENCY_SECONDS.observe(elapsed);
                if attempt > 1 {
                    info!("Batch inserted successfully after {} attempts in {:.3}s", attempt, elapsed);
                } else {
                    debug!("Batch inserted successfully in {:.3}s", elapsed);
                }
                // Only clear buffer on success
                buffer.clear();
                BATCH_SIZE.set(0.0);
                return;
            }
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    // Final failure after all retries
                    error!("Failed to insert batch after {} attempts: {}", MAX_RETRIES, e);
                    error!("CRITICAL: {} records will be dropped due to persistent DB failure", batch_len);
                    // Clear buffer to prevent blocking
                    buffer.clear();
                    BATCH_SIZE.set(0.0);
                    return;
                }

                // Retry with exponential backoff: 100ms, 200ms, 400ms
                let backoff_ms = 100 * 2_u64.pow(attempt - 1);
                error!("Failed to insert batch (attempt {}/{}): {}. Retrying in {}ms...", 
                       attempt, MAX_RETRIES, e, backoff_ms);
                
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }
}
