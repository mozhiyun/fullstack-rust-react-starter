use sqlx::PgPool;

use crate::middleware::RateLimiter;
use crate::rbac_cache::RbacCache;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub access_token_exp_minutes: i64,
    pub refresh_token_exp_days: i64,
    pub redis_url: Option<String>,
    pub rbac_cache: RbacCache,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn from_env(pool: PgPool, jwt_secret: String) -> Self {
        let access_token_exp_minutes = std::env::var("ACCESS_TOKEN_EXP_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);
        let refresh_token_exp_days = std::env::var("REFRESH_TOKEN_EXP_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7);
        let redis_url = std::env::var("REDIS_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            pool,
            jwt_secret,
            access_token_exp_minutes,
            refresh_token_exp_days,
            redis_url,
            rbac_cache: RbacCache::from_env(),
            rate_limiter: RateLimiter::from_env(),
        }
    }
}
