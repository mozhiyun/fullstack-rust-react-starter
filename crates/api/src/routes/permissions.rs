use axum::extract::State;
use axum::Json;
use domain::permission::Permission;
use infra::repos::permission_repo;

use crate::error::{ApiError, ApiErrorBody};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/permissions",
    operation_id = "list_permissions",
    tag = "permissions",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Vec<Permission>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Permission>>, ApiError> {
    let permissions = permission_repo::list_all(&state.pool).await?;
    Ok(Json(permissions))
}
