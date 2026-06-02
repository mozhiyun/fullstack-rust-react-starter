use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use domain::user::UserStatus;
use infra::repos::{permission_repo, user_repo};
use moka::sync::Cache;
use sqlx::PgPool;
use uuid::Uuid;

use domain::DomainError;

/// RBAC / 用户状态短 TTL 缓存，减轻高频鉴权查库。
#[derive(Clone)]
pub struct RbacCache {
    permissions: Cache<Uuid, Arc<HashSet<String>>>,
    status: Cache<Uuid, UserStatus>,
}

impl RbacCache {
    pub fn from_env() -> Self {
        let ttl_secs = std::env::var("RBAC_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let max_capacity = std::env::var("RBAC_CACHE_MAX_ENTRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000);

        let ttl = Duration::from_secs(ttl_secs);
        Self {
            permissions: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(max_capacity)
                .build(),
            status: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(max_capacity)
                .build(),
        }
    }

    pub fn invalidate_user(&self, user_id: Uuid) {
        self.permissions.invalidate(&user_id);
        self.status.invalidate(&user_id);
    }

    pub async fn user_status(&self, pool: &PgPool, user_id: Uuid) -> Result<UserStatus, DomainError> {
        if let Some(status) = self.status.get(&user_id) {
            return Ok(status);
        }
        let status = user_repo::status_for_id(pool, user_id).await?;
        self.status.insert(user_id, status);
        Ok(status)
    }

    pub async fn permission_codes(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<String>, DomainError> {
        let set = self.permission_set(pool, user_id).await?;
        Ok(set.iter().cloned().collect())
    }

    pub async fn user_has_permission(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        code: &str,
    ) -> Result<bool, DomainError> {
        let set = self.permission_set(pool, user_id).await?;
        Ok(set.contains(code))
    }

    async fn permission_set(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Arc<HashSet<String>>, DomainError> {
        if let Some(cached) = self.permissions.get(&user_id) {
            return Ok(cached);
        }
        let codes = permission_repo::codes_for_user(pool, user_id).await?;
        let set = Arc::new(codes.into_iter().collect());
        self.permissions.insert(user_id, Arc::clone(&set));
        Ok(set)
    }
}
