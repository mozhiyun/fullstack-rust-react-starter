use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use domain::user::{CreateUser, UpdateUser, UserPublic};
use infra::repos::{role_repo, user_repo};
use uuid::Uuid;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::error::{ApiError, ApiErrorBody};
use crate::middleware::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// 每页条数（1–100）
    #[serde(default = "default_limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: i64,
    #[serde(default)]
    #[param(minimum = 0)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = UserPublic),
        (status = 401, body = ApiErrorBody),
    )
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<UserPublic>, ApiError> {
    let user = user_repo::find_by_id(&state.pool, auth.user_id).await?;
    Ok(Json(UserPublic::from(user)))
}

#[utoipa::path(
    get,
    path = "/api/users",
    operation_id = "list_users",
    tag = "users",
    params(ListQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Vec<UserPublic>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<UserPublic>>, ApiError> {
    let users = user_repo::list(&state.pool, q.limit, q.offset).await?;
    Ok(Json(users))
}

#[utoipa::path(
    get,
    path = "/api/users/{user_id}",
    operation_id = "get_user",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = UserPublic),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPublic>, ApiError> {
    let user = user_repo::find_by_id(&state.pool, user_id).await?;
    Ok(Json(UserPublic::from(user)))
}

#[utoipa::path(
    post,
    path = "/api/users",
    operation_id = "create_user",
    tag = "users",
    request_body = CreateUser,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "创建成功", body = UserPublic),
        (status = 400, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 409, body = ApiErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateUser>,
) -> Result<(axum::http::StatusCode, Json<UserPublic>), ApiError> {
    let user = user_repo::create(&state.pool, body).await?;

    if let Ok(user_role) = role_repo::find_by_code(&state.pool, "user").await {
        let _ = role_repo::assign_to_user(&state.pool, user.id, user_role.id).await;
    }

    Ok((axum::http::StatusCode::CREATED, Json(UserPublic::from(user))))
}

#[utoipa::path(
    patch,
    path = "/api/users/{user_id}",
    operation_id = "update_user",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    request_body = UpdateUser,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = UserPublic),
        (status = 400, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateUser>,
) -> Result<Json<UserPublic>, ApiError> {
    let user = user_repo::update(
        &state.pool,
        user_id,
        body.display_name,
        body.status,
    )
    .await?;
    state.rbac_cache.invalidate_user(user_id);
    Ok(Json(user))
}
