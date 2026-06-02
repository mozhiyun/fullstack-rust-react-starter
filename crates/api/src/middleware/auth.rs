use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::{Extension, Router};

use domain::user::UserStatus;

use crate::error::ApiError;
use crate::jwt;
use crate::state::AppState;

/// 路由上通过 Extension 声明所需权限 code（如 `users:read`）。
#[derive(Clone, Copy)]
pub struct RequiredPermission(pub &'static str);

#[derive(Clone, Copy)]
pub struct RequestUserId(pub uuid::Uuid);

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub email: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer(req.headers().get(AUTHORIZATION))
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "missing token"))?;

    let claims = jwt::verify(&state.jwt_secret, token)
        .map_err(|_| ApiError::new(StatusCode::UNAUTHORIZED, "invalid token"))?;

    let status = state.rbac_cache.user_status(&state.pool, claims.sub).await?;
    if status != UserStatus::Active {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "account disabled",
        ));
    }

    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
        email: claims.email,
    });

    let mut response = next.run(req).await;
    response.extensions_mut().insert(RequestUserId(claims.sub));
    Ok(response)
}

/// 在 `require_auth` 之后执行，读取 `RequiredPermission` 并查 RBAC 表（带短缓存）。
pub async fn require_permission(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(required): Extension<RequiredPermission>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let allowed = state
        .rbac_cache
        .user_has_permission(&state.pool, auth.user_id, required.0)
        .await?;

    if !allowed {
        return Err(ApiError::from(domain::DomainError::Forbidden));
    }

    Ok(next.run(req).await)
}

/// 仅需 JWT，不做 RBAC（如 `/api/users/me`）。
pub fn route_with_auth(state: &AppState, router: Router<AppState>) -> Router<AppState> {
    router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        require_auth,
    ))
}

/// 为子路由挂上 JWT 校验 + 权限声明 + RBAC 校验。
/// 注意：父级 `Router::layer(require_auth)` 对 `merge` 进来的子路由不生效，须在此一并挂载。
pub fn route_with_permission(
    state: &AppState,
    permission: &'static str,
    router: Router<AppState>,
) -> Router<AppState> {
    router
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_permission,
        ))
        .layer(Extension(RequiredPermission(permission)))
        .layer(axum::middleware::from_fn_with_state(state.clone(), require_auth))
}

pub fn extract_bearer(header: Option<&axum::http::HeaderValue>) -> Option<&str> {
    let value = header?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    Some(token.trim())
}
