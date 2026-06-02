use axum::extract::{Path, State};
use axum::Json;
use domain::permission::Permission;
use domain::role::{AssignRoleRequest, Role};
use infra::repos::{permission_repo, role_repo, user_repo};
use uuid::Uuid;

use crate::error::{ApiError, ApiErrorBody};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/users/{user_id}/roles",
    operation_id = "list_user_roles",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Vec<Role>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Role>>, ApiError> {
    user_repo::find_by_id(&state.pool, user_id).await?;
    let roles = role_repo::list_for_user(&state.pool, user_id).await?;
    Ok(Json(roles))
}

#[utoipa::path(
    get,
    path = "/api/users/{user_id}/permissions",
    operation_id = "list_user_permissions",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
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
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Permission>>, ApiError> {
    user_repo::find_by_id(&state.pool, user_id).await?;
    let permissions = permission_repo::list_for_user(&state.pool, user_id).await?;
    Ok(Json(permissions))
}

#[utoipa::path(
    post,
    path = "/api/users/{user_id}/roles",
    operation_id = "assign_user_role",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    request_body = AssignRoleRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Role),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn assign(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<AssignRoleRequest>,
) -> Result<Json<Role>, ApiError> {
    user_repo::find_by_id(&state.pool, user_id).await?;
    let role = role_repo::find_by_id(&state.pool, body.role_id).await?;
    role_repo::assign_to_user(&state.pool, user_id, body.role_id).await?;
    state.rbac_cache.invalidate_user(user_id);
    Ok(Json(role))
}

#[utoipa::path(
    delete,
    path = "/api/users/{user_id}/roles/{role_id}",
    operation_id = "remove_user_role",
    tag = "users",
    params(
        ("user_id" = Uuid, Path, description = "用户 ID"),
        ("role_id" = Uuid, Path, description = "角色 ID"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "已移除"),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn remove(
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    user_repo::find_by_id(&state.pool, user_id).await?;
    role_repo::find_by_id(&state.pool, role_id).await?;
    role_repo::remove_from_user(&state.pool, user_id, role_id).await?;
    state.rbac_cache.invalidate_user(user_id);
    Ok(axum::http::StatusCode::NO_CONTENT)
}
