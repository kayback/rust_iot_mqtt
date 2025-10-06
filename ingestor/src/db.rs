use crate::errors::Result;
use crate::metrics::DB_FAILURES_TOTAL;
use crate::model::Telemetry;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn make_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    info!("Database connection established");
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| crate::errors::Error::Database(sqlx::Error::Migrate(Box::new(e))))?;
    info!("Migrations completed");

    Ok(pool)
}

pub async fn insert_batch(pool: &PgPool, batch: &[Telemetry]) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        attempts += 1;
        match insert_batch_inner(pool, batch).await {
            Ok(()) => return Ok(()),
            Err(e) => match &e {
                crate::errors::Error::Database(db_err) => {
                    if attempts >= max_attempts || !is_transient_error(db_err) {
                        error!(
                            "Database insert failed permanently after {} attempts: {}",
                            attempts, e
                        );
                        return Err(e);
                    }

                    let wait_ms = 100 * 2_u64.pow(attempts - 1).min(32);
                    warn!(
                        "Database insert failed (attempt {}/{}), retrying in {}ms: {}",
                        attempts, max_attempts, wait_ms, db_err
                    );
                    DB_FAILURES_TOTAL.inc();
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                }
                _ => {
                    error!("Database insert failed with non-database error: {}", e);
                    return Err(e);
                }
            },
        }
    }
}

async fn insert_batch_inner(pool: &PgPool, batch: &[Telemetry]) -> Result<()> {
    let device_ids: Vec<&str> = batch.iter().map(|t| t.device_id.as_str()).collect();
    let timestamps: Vec<chrono::DateTime<chrono::Utc>> =
        batch.iter().map(|t| t.timestamp).collect();
    let temperatures: Vec<f64> = batch.iter().map(|t| t.temperature).collect();
    let humidities: Vec<f64> = batch.iter().map(|t| t.humidity).collect();
    let batteries: Vec<f64> = batch.iter().map(|t| t.battery).collect();

    let query = r#"
        INSERT INTO telemetry (device_id, ts, temperature, humidity, battery)
        SELECT * FROM UNNEST($1::text[], $2::timestamptz[], $3::float8[], $4::float8[], $5::float8[])
        ON CONFLICT (device_id, ts) DO NOTHING
        "#;

    sqlx::query(&query)
        .bind(&device_ids)
        .bind(&timestamps)
        .bind(&temperatures)
        .bind(&humidities)
        .bind(&batteries)
        .execute(pool)
        .await?;

    Ok(())
}

fn is_transient_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::PoolTimedOut | sqlx::Error::Io(_) | sqlx::Error::PoolClosed => true,
        sqlx::Error::Database(db_err) => {
            // Check if it's a connection-related error
            db_err.code().is_some_and(|code| {
                code == "08000" || // connection_exception
                code == "08003" || // connection_does_not_exist
                code == "08006" || // connection_failure
                code == "57P03" || // cannot_connect_now
                code == "53300" // too_many_connections
            })
        }
        _ => false,
    }
}
