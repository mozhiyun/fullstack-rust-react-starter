use chrono::{Duration, Utc};
use domain::DomainError;
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_raw_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("{}_{}", Uuid::new_v4(), hex::encode(bytes))
}

pub async fn issue(
    pool: &PgPool,
    user_id: Uuid,
    exp_days: i64,
) -> Result<String, DomainError> {
    let raw = generate_raw_token();
    let token_hash = hash_token(&raw);
    let expires_at = Utc::now() + Duration::days(exp_days.max(1));

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    Ok(raw)
}

pub async fn consume_and_rotate(
    pool: &PgPool,
    raw_token: &str,
    exp_days: i64,
) -> Result<(Uuid, String), DomainError> {
    let token_hash = hash_token(raw_token);
    let now = Utc::now();

    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked_at IS NULL
          AND expires_at > $2
        "#,
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    let (user_id,) = row.ok_or(DomainError::Unauthorized)?;

    sqlx::query(
        r#"UPDATE refresh_tokens SET revoked_at = $2 WHERE token_hash = $1"#,
    )
    .bind(&token_hash)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    let new_raw = issue(pool, user_id, exp_days).await?;
    Ok((user_id, new_raw))
}

pub async fn revoke(pool: &PgPool, raw_token: &str) -> Result<(), DomainError> {
    let token_hash = hash_token(raw_token);
    let now = Utc::now();

    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens SET revoked_at = $2
        WHERE token_hash = $1 AND revoked_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DomainError::Validation(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DomainError::Unauthorized);
    }
    Ok(())
}
