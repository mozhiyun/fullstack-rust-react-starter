use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::error::ApiErrorBody;
use crate::routes::health::HealthResponse;
use domain::auth::{LoginRequest, LoginResponse, LogoutRequest, RefreshRequest};
use domain::permission::Permission;
use domain::role::{AssignRoleRequest, Role};
use domain::user::{CreateUser, UpdateUser, UserPublic};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health,
        crate::routes::auth::login,
        crate::routes::auth::admin_login,
        crate::routes::auth::register,
        crate::routes::auth::refresh,
        crate::routes::auth::admin_refresh,
        crate::routes::auth::logout,
        crate::routes::users::me,
        crate::routes::users::list,
        crate::routes::users::get,
        crate::routes::users::create,
        crate::routes::users::update,
        crate::routes::user_roles::list,
        crate::routes::user_roles::list_permissions,
        crate::routes::user_roles::assign,
        crate::routes::user_roles::remove,
        crate::routes::roles::list,
        crate::routes::roles::list_permissions,
        crate::routes::permissions::list,
    ),
    components(
        schemas(
            HealthResponse,
            crate::routes::health::ComponentHealth,
            ApiErrorBody,
            LoginRequest,
            LoginResponse,
            RefreshRequest,
            LogoutRequest,
            CreateUser,
            UpdateUser,
            UserPublic,
            domain::user::UserStatus,
            Role,
            Permission,
            AssignRoleRequest,
            crate::routes::users::ListQuery,
        )
    ),
    modifiers(&SecurityAddon, &InfoAddon),
    tags(
        (name = "health", description = "健康检查"),
        (name = "auth", description = "认证"),
        (name = "users", description = "用户"),
        (name = "roles", description = "角色"),
        (name = "permissions", description = "权限"),
    )
)]
pub struct ApiDoc;

struct InfoAddon;

impl Modify for InfoAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.title = "Fullstack Rust React Starter API".to_string();
        openapi.info.description =
            Some("Fullstack Rust React Starter — Axum + React monorepo".to_string());
    }
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
