pub mod auth;
pub mod health;
pub mod permissions;
pub mod roles;
pub mod user_roles;
pub mod users;

use axum::routing::{delete, get, patch, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::middleware::{apply_global_layers, route_with_auth, route_with_permission};
use crate::openapi::ApiDoc;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(health::health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/admin/login", post(auth::admin_login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh))
        .route("/api/auth/admin/refresh", post(auth::admin_refresh))
        .route("/api/auth/logout", post(auth::logout));

    let users_me = route_with_auth(
        &state,
        Router::new().route("/api/users/me", get(users::me)),
    );

    let users_read = route_with_permission(
        &state,
        "users:read",
        Router::new()
            .route("/api/users", get(users::list))
            .route("/api/users/{user_id}", get(users::get)),
    );

    let users_write = route_with_permission(
        &state,
        "users:write",
        Router::new()
            .route("/api/users", post(users::create))
            .route("/api/users/{user_id}", patch(users::update)),
    );

    let user_roles_read = route_with_permission(
        &state,
        "roles:read",
        Router::new()
            .route("/api/users/{user_id}/roles", get(user_roles::list))
            .route(
                "/api/users/{user_id}/permissions",
                get(user_roles::list_permissions),
            ),
    );

    let user_roles_write = route_with_permission(
        &state,
        "roles:write",
        Router::new()
            .route("/api/users/{user_id}/roles", post(user_roles::assign))
            .route(
                "/api/users/{user_id}/roles/{role_id}",
                delete(user_roles::remove),
            ),
    );

    let roles_list = route_with_permission(
        &state,
        "roles:read",
        Router::new()
            .route("/api/roles", get(roles::list))
            .route(
                "/api/roles/{role_id}/permissions",
                get(roles::list_permissions),
            ),
    );

    let permissions_list = route_with_permission(
        &state,
        "roles:read",
        Router::new().route("/api/permissions", get(permissions::list)),
    );

    let authed = Router::new()
        .merge(users_me)
        .merge(users_read)
        .merge(users_write)
        .merge(user_roles_read)
        .merge(user_roles_write)
        .merge(roles_list)
        .merge(permissions_list);

    let openapi = Router::new().merge(
        SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()),
    );

    let app = Router::new()
        .merge(public)
        .merge(authed)
        .merge(openapi);

    apply_global_layers(app, &state).with_state(state)
}
