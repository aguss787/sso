pub mod jwt;

use crate::helpers::InternalError;
use crate::kvs::KvsPool;
use crate::services::tokens::jwt::{Claims, JwtSigner, JwtType, JwtVerifyError};
use redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use std::sync::Arc;

pub struct JwtSecret<'a>(pub &'a [u8]);

pub struct TokenService {
    kv_pool: Arc<KvsPool>,
    jwt_signer: JwtSigner,
}

impl TokenService {
    pub fn new(kv_pool: Arc<KvsPool>, JwtSecret(secret): JwtSecret) -> Self {
        Self {
            kv_pool,
            jwt_signer: JwtSigner::new(secret),
        }
    }
}

impl TokenService {
    pub fn verify_any(&self, token: &str) -> Result<Claims, JwtVerifyError> {
        self.jwt_signer.verify(token)
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims, JwtVerifyError> {
        let claims = self.verify_any(token)?;
        if claims.jwt_type != JwtType::AccessToken {
            return Err(JwtVerifyError::InvalidToken);
        }

        Ok(claims)
    }

    pub fn verify_activation_code(&self, token: &str) -> Result<Claims, JwtVerifyError> {
        let claims = self.verify_any(token)?;
        if claims.jwt_type != JwtType::ActivationCode {
            return Err(JwtVerifyError::InvalidToken);
        }

        Ok(claims)
    }

    pub fn create_authorization_code(
        &self,
        client_id: String,
        user_id: uuid::Uuid,
        expiry: chrono::Duration,
    ) -> Result<String, InternalError> {
        self.jwt_signer.sign(&Claims::new(
            JwtType::AuthorizationCode,
            client_id,
            user_id,
            expiry,
        ))
    }

    pub fn create_access_token(
        &self,
        client_id: String,
        user_id: uuid::Uuid,
        expiry: chrono::Duration,
    ) -> Result<String, InternalError> {
        self.jwt_signer.sign(&Claims::new(
            JwtType::AccessToken,
            client_id,
            user_id,
            expiry,
        ))
    }

    pub fn create_activation_code(&self, user_id: uuid::Uuid) -> Result<String, InternalError> {
        self.jwt_signer.sign(&Claims::new(
            JwtType::ActivationCode,
            "agus.dev sso".to_string(),
            user_id,
            chrono::Duration::minutes(15),
        ))
    }

    pub async fn mark_authorization_code_as_used(
        &self,
        token: &str,
    ) -> Result<bool, InternalError> {
        let mut conn = self.kv_pool.get().await?;
        let key = format!("authorization_token:{}", token);

        let result: Option<String> = conn
            .set_options(
                &key,
                token,
                SetOptions::default()
                    .conditional_set(ExistenceCheck::NX)
                    .with_expiration(SetExpiry::EX(60 * 5)),
            )
            .await?;

        Ok(result.is_some())
    }
}
