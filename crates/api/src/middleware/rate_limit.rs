use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, Bucket>>,
    auth_max: u32,
    global_max: u32,
    window: Duration,
}

#[derive(Debug)]
struct Bucket {
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        let auth_max = std::env::var("RATE_LIMIT_AUTH_PER_MIN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);
        let global_max = std::env::var("RATE_LIMIT_GLOBAL_PER_MIN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        Self {
            buckets: Arc::new(DashMap::new()),
            auth_max,
            global_max,
            window: Duration::from_secs(60),
        }
    }

    fn check(&self, key: &str, path: &str) -> Result<(), ApiError> {
        let limit = if is_auth_path(path) {
            self.auth_max
        } else {
            self.global_max
        };

        let now = Instant::now();
        let mut exceeded = false;

        self.buckets
            .entry(key.to_string())
            .and_modify(|bucket| {
                if now.duration_since(bucket.window_start) >= self.window {
                    bucket.window_start = now;
                    bucket.count = 1;
                } else {
                    bucket.count += 1;
                    if bucket.count > limit {
                        exceeded = true;
                    }
                }
            })
            .or_insert_with(|| Bucket {
                window_start: now,
                count: 1,
            });

        if exceeded {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate limit exceeded",
            ));
        }
        Ok(())
    }
}

fn is_auth_path(path: &str) -> bool {
    path.starts_with("/api/auth/")
}

fn client_key(req: &Request) -> String {
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(ip) = forwarded.split(',').next().map(str::trim) {
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }
    if let Some(real) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        if !real.is_empty() {
            return real.to_string();
        }
    }
    "unknown".to_string()
}

pub async fn rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let key = client_key(&req);

    if let Err(err) = state.rate_limiter.check(&key, &path) {
        return err.into_response();
    }

    next.run(req).await
}
