use axum::extract::{Path, State};
use axum::Json;
use domain::permission::Permission;
use domain::role::Role;
use infra::repos::{permission_repo, role_repo};
use uuid::Uuid;

use crate::error::{ApiError, ApiErrorBody};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/roles",
    operation_id = "list_roles",
    tag = "roles",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Vec<Role>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Role>>, ApiError> {
    let roles = role_repo::list_all(&state.pool).await?;
    Ok(Json(roles))
}

#[utoipa::path(
    get,
    path = "/api/roles/{role_id}/permissions",
    operation_id = "list_role_permissions",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "角色 ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Vec<Permission>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn list_permissions(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> Result<Json<Vec<Permission>>, ApiError> {
    role_repo::find_by_id(&state.pool, role_id).await?;
    let permissions = permission_repo::list_for_role(&state.pool, role_id).await?;
    Ok(Json(permissions))
}
