use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::auth::{LoginRequest, LoginResponse, LogoutRequest, RefreshRequest};
use domain::user::{CreateUser, UserStatus};
use infra::password;
use infra::refresh_token;
use infra::repos::{role_repo, user_repo};

use crate::error::{ApiError, ApiErrorBody};
use crate::state::AppState;
use crate::tokens::issue_login_response;

/// 进入管理后台的最低权限（与侧边栏「角色/权限」一致；`user` 角色仅有 `users:read`，不能登录 admin）。
const ADMIN_ENTRY_PERMISSION: &str = "roles:read";

fn ensure_admin_panel_access(permissions: &[String]) -> Result<(), ApiError> {
    if permissions.iter().any(|p| p == ADMIN_ENTRY_PERMISSION) {
        return Ok(());
    }
    Err(ApiError::new(
        StatusCode::FORBIDDEN,
        "no permission to access admin panel",
    ))
}

async fn authenticate_credentials(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<domain::user::User, ApiError> {
    let user = user_repo::find_by_email(&state.pool, email)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "invalid credentials"))?;

    if !password::verify_password(password, &user.password_hash) {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "invalid credentials"));
    }

    if user.status != UserStatus::Active {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "account disabled"));
    }

    Ok(user)
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功", body = LoginResponse),
        (status = 401, description = "凭证错误", body = ApiErrorBody),
        (status = 403, description = "账号已禁用", body = ApiErrorBody),
        (status = 429, description = "请求过于频繁", body = ApiErrorBody),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = authenticate_credentials(&state, &body.email, &body.password).await?;
    Ok(Json(issue_login_response(&state, user, None).await?))
}

#[utoipa::path(
    post,
    path = "/api/auth/admin/login",
    operation_id = "admin_login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "管理后台登录成功", body = LoginResponse),
        (status = 401, description = "凭证错误", body = ApiErrorBody),
        (status = 403, description = "无管理后台权限或账号已禁用", body = ApiErrorBody),
        (status = 429, description = "请求过于频繁", body = ApiErrorBody),
    )
)]
pub async fn admin_login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = authenticate_credentials(&state, &body.email, &body.password).await?;
    let permissions = state
        .rbac_cache
        .permission_codes(&state.pool, user.id)
        .await?;
    ensure_admin_panel_access(&permissions)?;
    Ok(Json(issue_login_response(&state, user, None).await?))
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = CreateUser,
    responses(
        (status = 200, description = "注册并登录", body = LoginResponse),
        (status = 400, description = "校验失败", body = ApiErrorBody),
        (status = 409, description = "邮箱已存在", body = ApiErrorBody),
        (status = 429, description = "请求过于频繁", body = ApiErrorBody),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CreateUser>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = user_repo::create(&state.pool, body).await?;

    if let Ok(user_role) = role_repo::find_by_code(&state.pool, "user").await {
        let _ = role_repo::assign_to_user(&state.pool, user.id, user_role.id).await?;
    }
    state.rbac_cache.invalidate_user(user.id);

    Ok(Json(issue_login_response(&state, user, None).await?))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "刷新成功", body = LoginResponse),
        (status = 401, description = "refresh token 无效", body = ApiErrorBody),
        (status = 429, description = "请求过于频繁", body = ApiErrorBody),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (user_id, new_refresh) =
        refresh_token::consume_and_rotate(&state.pool, &body.refresh_token, state.refresh_token_exp_days)
            .await
            .map_err(ApiError::from)?;

    let user = user_repo::find_by_id(&state.pool, user_id).await?;

    if user.status != UserStatus::Active {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "account disabled"));
    }

    Ok(Json(
        issue_login_response(&state, user, Some(new_refresh)).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/admin/refresh",
    operation_id = "admin_refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "刷新成功", body = LoginResponse),
        (status = 401, description = "refresh token 无效", body = ApiErrorBody),
        (status = 403, description = "无管理后台权限或账号已禁用", body = ApiErrorBody),
        (status = 429, description = "请求过于频繁", body = ApiErrorBody),
    )
)]
pub async fn admin_refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (user_id, new_refresh) =
        refresh_token::consume_and_rotate(&state.pool, &body.refresh_token, state.refresh_token_exp_days)
            .await
            .map_err(ApiError::from)?;

    let user = user_repo::find_by_id(&state.pool, user_id).await?;

    if user.status != UserStatus::Active {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "account disabled"));
    }

    let permissions = state
        .rbac_cache
        .permission_codes(&state.pool, user.id)
        .await?;
    ensure_admin_panel_access(&permissions)?;

    Ok(Json(
        issue_login_response(&state, user, Some(new_refresh)).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "已登出"),
        (status = 401, description = "token 无效", body = ApiErrorBody),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    refresh_token::revoke(&state.pool, &body.refresh_token)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
