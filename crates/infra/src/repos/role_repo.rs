use domain::role::Role;
use domain::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_all(pool: &PgPool) -> Result<Vec<Role>, DomainError> {
    sqlx::query_as::<_, Role>(
        r#"SELECT id, code, name, description, created_at FROM roles ORDER BY code"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))
}

pub async fn find_by_id(pool: &PgPool, role_id: Uuid) -> Result<Role, DomainError> {
    sqlx::query_as::<_, Role>(
        r#"SELECT id, code, name, description, created_at FROM roles WHERE id = $1"#,
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)
}

pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Role, DomainError> {
    sqlx::query_as::<_, Role>(
        r#"SELECT id, code, name, description, created_at FROM roles WHERE code = $1"#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)
}

pub async fn assign_to_user(pool: &PgPool, user_id: Uuid, role_id: Uuid) -> Result<(), DomainError> {
    sqlx::query(
        r#"INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
    )
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;
    Ok(())
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Role>, DomainError> {
    sqlx::query_as::<_, Role>(
        r#"
        SELECT r.id, r.code, r.name, r.description, r.created_at
        FROM roles r
        INNER JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1
        ORDER BY r.code
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))
}

pub async fn remove_from_user(
    pool: &PgPool,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<(), DomainError> {
    let result = sqlx::query(
        r#"DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2"#,
    )
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DomainError::NotFound);
    }
    Ok(())
}

pub async fn codes_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, DomainError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT r.code FROM roles r
        INNER JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1
        ORDER BY r.code
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(rows.into_iter().map(|(c,)| c).collect())
}
