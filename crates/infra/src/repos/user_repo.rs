use domain::user::{CreateUser, User, UserPublic, UserStatus};
use domain::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::password;

pub async fn create(pool: &PgPool, input: CreateUser) -> Result<User, DomainError> {
    if input.email.trim().is_empty() || input.password.len() < 8 {
        return Err(DomainError::Validation(
            "email required and password min 8 chars".into(),
        ));
    }

    let password_hash = password::hash_password(&input.password)
        .map_err(|e| DomainError::Validation(e.to_string()))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, display_name, status, created_at, updated_at
        "#,
    )
    .bind(input.email.trim().to_lowercase())
    .bind(password_hash)
    .bind(input.display_name.trim())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.constraint().is_some() {
                return DomainError::Conflict("email already exists".into());
            }
        }
        DomainError::Validation(e.to_string())
    })?;

    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, DomainError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, display_name, status, created_at, updated_at
        FROM users WHERE email = $1
        "#,
    )
    .bind(email.trim().to_lowercase())
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(user)
}

/// 供鉴权中间件使用：只查状态，避免每次请求加载 password_hash。
pub async fn status_for_id(pool: &PgPool, id: Uuid) -> Result<UserStatus, DomainError> {
    let row: (UserStatus,) = sqlx::query_as(r#"SELECT status FROM users WHERE id = $1"#)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?
        .ok_or(DomainError::NotFound)?;

    Ok(row.0)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<User, DomainError> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, display_name, status, created_at, updated_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)
}

pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<UserPublic>, DomainError> {
    let rows = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, display_name, status, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit.clamp(1, 100))
    .bind(offset.max(0))
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(rows.into_iter().map(UserPublic::from).collect())
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    display_name: Option<String>,
    status: Option<UserStatus>,
) -> Result<UserPublic, DomainError> {
    let existing = find_by_id(pool, id).await?;

    let display_name = display_name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or(existing.display_name);
    let status = status.unwrap_or(existing.status);

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET display_name = $2, status = $3
        WHERE id = $1
        RETURNING id, email, password_hash, display_name, status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(display_name)
    .bind(status)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)?;

    Ok(UserPublic::from(user))
}

pub async fn set_status(pool: &PgPool, id: Uuid, status: UserStatus) -> Result<UserPublic, DomainError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET status = $2
        WHERE id = $1
        RETURNING id, email, password_hash, display_name, status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)?;

    Ok(UserPublic::from(user))
}
