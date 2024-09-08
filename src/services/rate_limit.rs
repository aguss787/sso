use crate::helpers::InternalError;
use crate::kvs::KvsPool;
use redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use std::sync::Arc;

pub struct RateLimitService {
    kvs_pool: Arc<KvsPool>,
}

impl RateLimitService {
    pub fn new(kvs_pool: Arc<KvsPool>) -> Self {
        Self { kvs_pool }
    }
}

impl RateLimitService {
    pub async fn check_rate_limit(
        &self,
        key: &str,
        ttl: chrono::Duration,
    ) -> Result<bool, InternalError> {
        let sec = ttl.num_seconds() as u64;

        let mut conn = self.kvs_pool.get().await?;
        let result: Option<String> = conn
            .set_options(
                key,
                key,
                SetOptions::default()
                    .conditional_set(ExistenceCheck::NX)
                    .with_expiration(SetExpiry::EX(sec)),
            )
            .await?;

        Ok(result.is_some())
    }
}
