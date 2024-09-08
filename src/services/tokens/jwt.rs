use crate::helpers::{InternalError, ManualErrorHandle, ManualErrorHandling};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JwtType {
    AuthorizationCode,
    AccessToken,
    RefreshToken,
    ActivationCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub jwt_type: JwtType,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub sub: Uuid,
}

impl Claims {
    pub(super) fn new(jwt_type: JwtType, aud: String, sub: Uuid, exp: chrono::Duration) -> Self {
        let iat = chrono::Utc::now().timestamp() as usize;
        let exp = iat + exp.num_seconds() as usize;

        Self {
            jwt_type,
            aud,
            exp,
            iat,
            iss: "agus.dev sso".to_string(),
            sub,
        }
    }
}

pub(super) struct JwtSigner {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtSigner {
    pub(super) fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub(super) fn sign(&self, claims: &Claims) -> Result<String, InternalError> {
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, claims, &self.encoding_key)?;
        Ok(token)
    }

    pub(super) fn verify(&self, token: &str) -> Result<Claims, JwtVerifyError> {
        let mut validation = jsonwebtoken::Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        validation.validate_exp = true;
        validation.set_issuer::<&str>(&["agus.dev sso"]);

        let token_data = jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &validation)
            .manual_error_handling()?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for JwtVerifyError {
    fn into_response(self) -> Response {
        match self {
            JwtVerifyError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "invalid token").into_response()
            }
            JwtVerifyError::ExpiredToken => {
                (StatusCode::UNAUTHORIZED, "expired token").into_response()
            }
            JwtVerifyError::InternalError(e) => e.into_response(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtVerifyError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Expired token")]
    ExpiredToken,
    #[error("Internal error: {0}")]
    InternalError(InternalError),
}

impl From<ManualErrorHandling<jsonwebtoken::errors::Error>> for JwtVerifyError {
    fn from(error: ManualErrorHandling<jsonwebtoken::errors::Error>) -> Self {
        let error = error.into_inner();
        match error.kind() {
            ErrorKind::InvalidToken => Self::InvalidToken,
            ErrorKind::InvalidSignature => Self::InvalidToken,
            ErrorKind::ExpiredSignature => Self::ExpiredToken,
            ErrorKind::InvalidIssuer => Self::InvalidToken,
            ErrorKind::InvalidAudience => Self::InvalidToken,
            ErrorKind::InvalidSubject => Self::InvalidToken,
            ErrorKind::ImmatureSignature => Self::InvalidToken,
            _ => Self::InternalError(error.into()),
        }
    }
}
