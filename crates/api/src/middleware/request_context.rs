use std::time::Instant;

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

use super::auth::RequestUserId;

/// 贯穿请求生命周期的 Request ID（也写入响应头 `X-Request-Id`）。
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn request_context(mut req: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    req.extensions_mut()
        .insert(RequestId(request_id.clone()));

    let response = next.run(req).await;
    let status = response.status();
    let latency_ms = started.elapsed().as_millis();
    let user_id = response.extensions().get::<RequestUserId>().map(|u| u.0);

    log_http_request(
        &request_id,
        &method,
        &uri,
        status,
        latency_ms,
        user_id,
    );

    let (mut parts, body) = response.into_parts();
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        parts.headers.insert(HeaderName::from_static("x-request-id"), value);
    }
    Response::from_parts(parts, body)
}

/// 访问日志级别：2xx/3xx → INFO，4xx → WARN，5xx → ERROR。
fn log_http_request(
    request_id: &str,
    method: &axum::http::Method,
    uri: &axum::http::Uri,
    status: axum::http::StatusCode,
    latency_ms: u128,
    user_id: Option<uuid::Uuid>,
) {
    let status_code = status.as_u16();
    let user_field = user_id.map(|id| tracing::field::display(id));

    if status.is_server_error() {
        tracing::error!(
            %request_id,
            %method,
            %uri,
            status = status_code,
            latency_ms,
            user_id = user_field,
            "http request"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            %request_id,
            %method,
            %uri,
            status = status_code,
            latency_ms,
            user_id = user_field,
            "http request"
        );
    } else {
        tracing::info!(
            %request_id,
            %method,
            %uri,
            status = status_code,
            latency_ms,
            user_id = user_field,
            "http request"
        );
    }
}
