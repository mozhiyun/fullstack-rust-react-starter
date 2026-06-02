use domain::auth::LoginResponse;
use domain::user::User;
use infra::refresh_token;
use infra::repos::role_repo;

use crate::error::ApiError;
use crate::jwt;
use crate::state::AppState;

pub async fn issue_login_response(
    state: &AppState,
    user: User,
    existing_refresh: Option<String>,
) -> Result<LoginResponse, ApiError> {
    let roles = role_repo::codes_for_user(&state.pool, user.id).await?;
    let permissions = state
        .rbac_cache
        .permission_codes(&state.pool, user.id)
        .await?;

    let access_token = jwt::issue_access(
        &state.jwt_secret,
        user.id,
        &user.email,
        state.access_token_exp_minutes,
    )
    .map_err(|e| ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token = match existing_refresh {
        Some(token) => token,
        None => refresh_token::issue(&state.pool, user.id, state.refresh_token_exp_days).await?,
    };

    Ok(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        roles,
        permissions,
    })
}
