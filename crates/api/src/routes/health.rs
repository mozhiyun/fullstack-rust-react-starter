use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct ComponentHealth {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub postgres: ComponentHealth,
    pub redis: ComponentHealth,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "服务正常或降级", body = HealthResponse),
        (status = 503, description = "依赖不可用", body = HealthResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> Response {
    let postgres = check_postgres(&state.pool).await;
    let redis = check_redis(state.redis_url.as_deref()).await;

    let postgres_ok = postgres.status == "ok";
    let overall = if postgres_ok {
        if redis.status == "error" {
            "degraded"
        } else {
            "ok"
        }
    } else {
        "unhealthy"
    };

    let body = HealthResponse {
        status: overall,
        postgres,
        redis,
    };

    let status_code = if postgres_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(body)).into_response()
}

async fn check_postgres(pool: &sqlx::PgPool) -> ComponentHealth {
    let started = Instant::now();
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => ComponentHealth {
            status: "ok",
            latency_ms: Some(started.elapsed().as_millis() as u64),
            message: None,
        },
        Err(e) => ComponentHealth {
            status: "error",
            latency_ms: None,
            message: Some(e.to_string()),
        },
    }
}

async fn check_redis(redis_url: Option<&str>) -> ComponentHealth {
    let Some(url) = redis_url else {
        return ComponentHealth {
            status: "skipped",
            latency_ms: None,
            message: Some("REDIS_URL not configured".into()),
        };
    };

    let started = Instant::now();
    match redis::Client::open(url) {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => match redis::cmd("PING").query_async::<String>(&mut conn).await {
                Ok(_) => ComponentHealth {
                    status: "ok",
                    latency_ms: Some(started.elapsed().as_millis() as u64),
                    message: None,
                },
                Err(e) => ComponentHealth {
                    status: "error",
                    latency_ms: None,
                    message: Some(e.to_string()),
                },
            },
            Err(e) => ComponentHealth {
                status: "error",
                latency_ms: None,
                message: Some(e.to_string()),
            },
        },
        Err(e) => ComponentHealth {
            status: "error",
            latency_ms: None,
            message: Some(e.to_string()),
        },
    }
}
