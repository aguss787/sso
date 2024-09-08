use crate::helpers::InternalError;
use crate::services::clients::{Client, ClientService};
use crate::services::tokens::jwt::{Claims, JwtType, JwtVerifyError};
use crate::services::tokens::TokenService;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub struct Oauth2Service {
    pub token_service: Arc<TokenService>,
    pub client_service: Arc<ClientService>,
}

#[derive(Deserialize)]
pub struct TokenParams {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize)]
pub struct AccessToken {
    access_token: String,
    token_type: &'static str,
    expires_in: usize,
    refresh_token: Option<String>,
    scope: Option<String>,
}

impl Oauth2Service {
    pub fn new(token_service: Arc<TokenService>, client_service: Arc<ClientService>) -> Self {
        Self {
            token_service,
            client_service,
        }
    }

    pub fn create_authorization_code(
        &self,
        client_id: String,
        user_id: Uuid,
    ) -> Result<String, InternalError> {
        let expiry = chrono::Duration::minutes(5);
        self.token_service
            .create_authorization_code(client_id, user_id, expiry)
    }

    pub async fn access_token(
        &self,
        token_params: &TokenParams,
    ) -> Result<AccessToken, AccessTokenError> {
        let client = self
            .client_service
            .get_by_client_id(&token_params.client_id)
            .await?
            .ok_or(AccessTokenError::ClientAuthenticationFailed)?;

        if !client.is_secret_match(&token_params.client_secret)? {
            tracing::warn!("mismatch client secret");
            return Err(AccessTokenError::ClientAuthenticationFailed);
        }

        let claims = self.token_service.verify_any(&token_params.code)?;
        match token_params.grant_type.as_str() {
            "authorization_code" => {
                self.authorization_code_flow(&claims, &client, token_params)
                    .await
            }
            _ => Err(AccessTokenError::UnsupportedGrantType),
        }
    }

    async fn authorization_code_flow(
        &self,
        claims: &Claims,
        client: &Client,
        token_params: &TokenParams,
    ) -> Result<AccessToken, AccessTokenError> {
        if claims.jwt_type != JwtType::AuthorizationCode {
            return Err(AccessTokenError::TokenTypeMismatch);
        }

        if claims.aud != token_params.client_id {
            return Err(AccessTokenError::TokenAudienceMismatch);
        }

        if token_params.redirect_uri != client.redirect_uri {
            return Err(AccessTokenError::RedirectUriMismatch);
        }

        if !self
            .token_service
            .mark_authorization_code_as_used(&token_params.code)
            .await?
        {
            return Err(AccessTokenError::AuthorizationCodeUsed);
        };

        let expiry = chrono::Duration::minutes(60);
        let token = self.token_service.create_access_token(
            token_params.client_id.clone(),
            claims.sub,
            expiry,
        )?;

        Ok(AccessToken {
            access_token: token,
            token_type: "Bearer",
            expires_in: expiry.num_seconds() as usize,
            refresh_token: None,
            scope: None,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccessTokenError {
    #[error("unsupported grant type")]
    UnsupportedGrantType,
    #[error("client authentication failed")]
    ClientAuthenticationFailed,
    #[error("token audience mismatch")]
    TokenAudienceMismatch,
    #[error("redirect uri mismatch")]
    RedirectUriMismatch,
    #[error("authorization code already used")]
    AuthorizationCodeUsed,
    #[error("token type mismatch")]
    TokenTypeMismatch,
    #[error("invalid token")]
    InvalidToken(#[from] JwtVerifyError),
    #[error("internal error: {0}")]
    InternalError(InternalError),
}

impl<T: Into<InternalError>> From<T> for AccessTokenError {
    fn from(error: T) -> Self {
        AccessTokenError::InternalError(error.into())
    }
}

#[derive(Serialize)]
pub struct OauthErrorResponse {
    error: &'static str,
    error_description: Option<&'static str>,
}

impl IntoResponse for AccessTokenError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenError::UnsupportedGrantType => (
                StatusCode::BAD_REQUEST,
                Json(OauthErrorResponse {
                    error: "unsupported_grant_type",
                    error_description: None,
                }),
            )
                .into_response(),
            AccessTokenError::ClientAuthenticationFailed => (
                StatusCode::UNAUTHORIZED,
                Json(OauthErrorResponse {
                    error: "invalid_client",
                    error_description: None,
                }),
            )
                .into_response(),
            AccessTokenError::TokenTypeMismatch => (
                StatusCode::BAD_REQUEST,
                Json(OauthErrorResponse {
                    error: "invalid_grant",
                    error_description: Some("token type mismatch"),
                }),
            )
                .into_response(),
            AccessTokenError::TokenAudienceMismatch => (
                StatusCode::BAD_REQUEST,
                Json(OauthErrorResponse {
                    error: "invalid_grant",
                    error_description: Some("token audience mismatch"),
                }),
            )
                .into_response(),
            AccessTokenError::RedirectUriMismatch => (
                StatusCode::BAD_REQUEST,
                Json(OauthErrorResponse {
                    error: "invalid_grant",
                    error_description: Some("redirect uri mismatch"),
                }),
            )
                .into_response(),
            AccessTokenError::AuthorizationCodeUsed => (
                StatusCode::BAD_REQUEST,
                Json(OauthErrorResponse {
                    error: "invalid_grant",
                    error_description: Some("authorization code already used"),
                }),
            )
                .into_response(),
            AccessTokenError::InvalidToken(e) => match e {
                JwtVerifyError::InvalidToken => (
                    StatusCode::BAD_REQUEST,
                    Json(OauthErrorResponse {
                        error: "invalid_grant",
                        error_description: Some("invalid token"),
                    }),
                )
                    .into_response(),
                JwtVerifyError::ExpiredToken => (
                    StatusCode::BAD_REQUEST,
                    Json(OauthErrorResponse {
                        error: "invalid_grant",
                        error_description: Some("expired token"),
                    }),
                )
                    .into_response(),
                JwtVerifyError::InternalError(e) => e.into_response(),
            },
            AccessTokenError::InternalError(e) => e.into_response(),
        }
    }
}
