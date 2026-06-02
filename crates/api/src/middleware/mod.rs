mod auth;
mod rate_limit;
mod request_context;

pub use auth::{
    extract_bearer, route_with_auth, route_with_permission, require_auth, require_permission,
    AuthUser, RequiredPermission,
};
pub use rate_limit::{rate_limit, RateLimiter};
pub use request_context::{request_context, RequestId};

use std::env;

use axum::http::{header, HeaderName, HeaderValue};
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::state::AppState;

/// 全局 HTTP 中间件（最后 `with_state` 前挂载；由外到内：Request ID + 日志 → 限流 → 安全头 → Body 限制 → CORS）。
pub fn apply_global_layers<S>(router: Router<S>, state: &AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let body_limit_mb = env::var("HTTP_BODY_LIMIT_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    let body_limit = (body_limit_mb as usize) * 1024 * 1024;

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    router
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit,
        ))
        .layer(axum::middleware::from_fn(request_context::request_context))
}
