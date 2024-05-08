use crate::db::DbPoolError;
use crate::kvs::KvsPoolError;
use async_trait::async_trait;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::ops::Deref;

pub trait Validatable {
    type Rejection;

    fn validate(&self) -> Result<(), Self::Rejection> {
        Ok(())
    }
}

pub enum ValidationRejection<T, V> {
    ValidationFailed(T),
    FromRequestRejection(V),
}

impl<T, V> IntoResponse for ValidationRejection<T, V>
where
    T: IntoResponse,
    V: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::ValidationFailed(rejection) => rejection.into_response(),
            ValidationRejection::FromRequestRejection(rejection) => rejection.into_response(),
        }
    }
}

pub struct Validate<T>(pub T);

#[async_trait]
impl<S, T, D> FromRequest<S> for Validate<T>
where
    S: Send + Sync,
    T: FromRequest<S>,
    T: Deref<Target = D> + 'static,
    D: Validatable,
    D::Rejection: IntoResponse,
{
    type Rejection = ValidationRejection<D::Rejection, T::Rejection>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        T::from_request(req, state)
            .await
            .map_err(ValidationRejection::FromRequestRejection)
            .and_then(|o| match o.validate() {
                Ok(_) => Ok(o),
                Err(rejection) => Err(ValidationRejection::ValidationFailed(rejection)),
            })
            .map(Validate)
    }
}

pub struct ManualErrorHandling<T>(pub T);

impl<T> ManualErrorHandling<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ManualErrorHandling<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ManualErrorHandle {
    type Value;
    type Error;

    fn manual_error_handling(self) -> Result<Self::Value, ManualErrorHandling<Self::Error>>;
}

impl<T, E> ManualErrorHandle for Result<T, E> {
    type Value = T;
    type Error = E;

    fn manual_error_handling(self) -> Result<Self::Value, ManualErrorHandling<Self::Error>> {
        self.map_err(ManualErrorHandling)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("connection pool error: {0}")]
    ConnectionPool(#[from] DbPoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("password hash error: {0}")]
    PasswordHash(String),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("kvs pool error: {0}")]
    R2d2(#[from] KvsPoolError),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("lettre error: {0}")]
    Lettre(#[from] lettre::error::Error),

    #[error("lettre smtp error: {0}")]
    LettreSmtp(#[from] lettre::transport::smtp::Error),
}

impl From<argon2::password_hash::Error> for InternalError {
    fn from(err: argon2::password_hash::Error) -> Self {
        InternalError::PasswordHash(err.to_string())
    }
}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "internal error");
        (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
    }
}

pub struct TokenHeader(String);

#[async_trait]
impl<S> FromRequestParts<S> for TokenHeader {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, _states: &S) -> Result<Self, Self::Rejection> {
        req.headers
            .get("Authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "missing Authorization header"))
            .and_then(|header| {
                header
                    .to_str()
                    .map_err(|_| (StatusCode::BAD_REQUEST, "invalid Authorization header"))
            })
            .map(ToString::to_string)
            .map(Self)
    }
}

impl TokenHeader {
    pub fn to_bearer_token(&self) -> Result<&str, Response> {
        if !self.0.starts_with("Bearer ") {
            return Err((StatusCode::BAD_REQUEST, "invalid Authorization header").into_response());
        }

        Ok(&self.0[7..])
    }
}
