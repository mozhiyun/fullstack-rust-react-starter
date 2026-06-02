use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use domain::error::DomainError;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorBody {
    pub error: String,
}

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound => ApiError::new(StatusCode::NOT_FOUND, "not found"),
            DomainError::Conflict(msg) => ApiError::new(StatusCode::CONFLICT, msg),
            DomainError::Unauthorized => ApiError::new(StatusCode::UNAUTHORIZED, "unauthorized"),
            DomainError::Forbidden => ApiError::new(StatusCode::FORBIDDEN, "forbidden"),
            DomainError::Validation(msg) => ApiError::new(StatusCode::BAD_REQUEST, msg),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ApiErrorBody {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}
