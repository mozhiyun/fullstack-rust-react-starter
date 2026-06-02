use domain::permission::Permission;
use domain::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_all(pool: &PgPool) -> Result<Vec<Permission>, DomainError> {
    sqlx::query_as::<_, Permission>(
        r#"SELECT id, code, name, description, created_at FROM permissions ORDER BY code"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))
}

pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Permission, DomainError> {
    sqlx::query_as::<_, Permission>(
        r#"SELECT id, code, name, description, created_at FROM permissions WHERE code = $1"#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?
    .ok_or(DomainError::NotFound)
}

pub async fn assign_to_role(pool: &PgPool, role_id: Uuid, permission_id: Uuid) -> Result<(), DomainError> {
    sqlx::query(
        r#"INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;
    Ok(())
}

pub async fn list_for_role(pool: &PgPool, role_id: Uuid) -> Result<Vec<Permission>, DomainError> {
    sqlx::query_as::<_, Permission>(
        r#"
        SELECT p.id, p.code, p.name, p.description, p.created_at
        FROM permissions p
        INNER JOIN role_permissions rp ON rp.permission_id = p.id
        WHERE rp.role_id = $1
        ORDER BY p.code
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Permission>, DomainError> {
    sqlx::query_as::<_, Permission>(
        r#"
        SELECT DISTINCT p.id, p.code, p.name, p.description, p.created_at
        FROM permissions p
        INNER JOIN role_permissions rp ON rp.permission_id = p.id
        INNER JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = $1
        ORDER BY p.code
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))
}

pub async fn codes_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, DomainError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT p.code FROM permissions p
        INNER JOIN role_permissions rp ON rp.permission_id = p.id
        INNER JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = $1
        ORDER BY p.code
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(rows.into_iter().map(|(c,)| c).collect())
}

pub async fn user_has_permission(pool: &PgPool, user_id: Uuid, code: &str) -> Result<bool, DomainError> {
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM permissions p
            INNER JOIN role_permissions rp ON rp.permission_id = p.id
            INNER JOIN user_roles ur ON ur.role_id = rp.role_id
            WHERE ur.user_id = $1 AND p.code = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(code)
    .fetch_one(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(row.0)
}
