use crate::model::{Telemetry, TelemetryResponse};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::error;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryQuery {
    device_id: Option<String>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub fn create_router(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/api/v1/telemetry", get(get_telemetry))
        .with_state(state)
}

async fn get_telemetry(
    State(state): State<AppState>,
    Query(params): Query<TelemetryQuery>,
) -> Result<Json<TelemetryResponse>, AppError> {
    let limit = params.limit.unwrap_or(100).min(1000);
    let offset = params.offset.unwrap_or(0);

    // Build query with filters
    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    // Device ID filter
    if let Some(device_id) = &params.device_id {
        conditions.push(format!("device_id = ${}", bind_values.len() + 1));
        bind_values.push(device_id.clone());
    }

    // Start time filter
    if let Some(start) = &params.start {
        conditions.push(format!("ts >= ${}", bind_values.len() + 1));
        bind_values.push(start.to_rfc3339());
    }

    // End time filter
    if let Some(end) = &params.end {
        conditions.push(format!("ts <= ${}", bind_values.len() + 1));
        bind_values.push(end.to_rfc3339());
    }

    // Build WHERE clause
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Build complete query
    let query = format!(
        "SELECT device_id, ts as timestamp, temperature, humidity, battery 
         FROM telemetry 
         {} 
         ORDER BY ts DESC 
         LIMIT {} OFFSET {}",
        where_clause, limit, offset
    );

    // Execute query with bindings
    let mut query_builder = sqlx::query_as::<_, Telemetry>(&query);
    
    // Bind parameters
    if let Some(device_id) = &params.device_id {
        query_builder = query_builder.bind(device_id);
    }
    if let Some(start) = &params.start {
        query_builder = query_builder.bind(start);
    }
    if let Some(end) = &params.end {
        query_builder = query_builder.bind(end);
    }

    let telemetry = query_builder
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError(anyhow::anyhow!("Database query failed: {}", e))
        })?;

    Ok(Json(TelemetryResponse {
        data: telemetry.clone(),
        total: telemetry.len(),
        limit,
        offset,
    }))
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("API error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
