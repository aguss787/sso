use crate::helpers::{TokenHeader, Validatable, Validate};
use crate::services::oauth2::{AccessToken, AccessTokenError, TokenParams};
use crate::services::users::{User, UserValidationError};
use crate::Services;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

impl Validatable for RegisterForm {
    type Rejection = (StatusCode, &'static str);

    fn validate(&self) -> Result<(), Self::Rejection> {
        if self.username.len() < 3 || self.username.len() > 32 {
            return Err((
                StatusCode::BAD_REQUEST,
                "username must be between 3 and 32 characters",
            ));
        }

        if !email_address::EmailAddress::is_valid(&self.email) {
            return Err((StatusCode::BAD_REQUEST, "invalid email address"));
        }

        if self.password.len() < 8 || self.password.len() > 32 {
            return Err((
                StatusCode::BAD_REQUEST,
                "password must be between 8 and 32 characters",
            ));
        }

        Ok(())
    }
}

pub async fn register(
    services: State<Arc<Services>>,
    Validate(Json(req)): Validate<Json<RegisterForm>>,
) -> Response {
    let user = match services
        .user_service
        .register(req.username, req.email, req.password)
        .await
    {
        Ok(user) => user,
        Err(e) => return e.into_response(),
    };

    let _ = generate_and_send_activation_email(services, user).inspect_err(|response| {
        tracing::error!(response = ?response, "failed to send activation email");
    });

    (StatusCode::CREATED, String::new()).into_response()
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    client_id: String,
    redirect_uri: String,
}

pub async fn login(
    services: State<Arc<Services>>,
    Form(req): Form<LoginForm>,
) -> Result<Redirect, Response> {
    // check for client_id and redirect_uri
    let client = services
        .client_service
        .get_by_client_id(&req.client_id)
        .await
        .map_err(IntoResponse::into_response)?
        .ok_or((StatusCode::BAD_REQUEST, "client_id is invalid").into_response())?;

    if req.redirect_uri != client.redirect_uri {
        tracing::info!(
            redirect_uri.expected = client.redirect_uri,
            redirect_uri.actual = req.redirect_uri,
            "redirect_uri does not match client's redirect_uri"
        );
        return Err((StatusCode::BAD_REQUEST, "redirect_uri mismatch").into_response());
    }

    let login_uri = |error| {
        Redirect::to(&format!(
            "/oauth2/login?error={}&client_id={}&redirect_uri={}",
            error, req.client_id, req.redirect_uri
        ))
        .into_response()
    };

    // check for password
    let user = match services
        .user_service
        .validate_and_return(&req.username, &req.password)
        .await
    {
        Ok(user) => Ok(user),
        Err(UserValidationError::UserNotFound) => {
            tracing::info!(username = &req.username, "user not found");
            Err(login_uri("invalid_credentials"))
        }
        Err(UserValidationError::InvalidPassword) => {
            tracing::info!(username = &req.username, "invalid password");
            Err(login_uri("invalid_credentials"))
        }
        Err(UserValidationError::NotActivated) => {
            tracing::info!(username = &req.username, "user not activated");
            Err(login_uri("not_activated"))
        }
        Err(UserValidationError::InternalError(_)) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "").into_response())
        }
    }?;

    // generate authorization code
    let auth_code = services
        .oauth2_service
        .create_authorization_code(req.client_id, user.id)
        .map_err(IntoResponse::into_response)?;

    let redirect_url = url::Url::parse_with_params(&req.redirect_uri, &[("code", &auth_code)])
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("invalid redirect url: {}", e),
            )
                .into_response()
        })?;
    Ok(Redirect::to(redirect_url.as_ref()))
}

pub async fn token(
    services: State<Arc<Services>>,
    token_form: Form<TokenParams>,
) -> Result<Json<AccessToken>, AccessTokenError> {
    Ok(Json(
        services.oauth2_service.access_token(&token_form).await?,
    ))
}

#[derive(Serialize)]
pub struct Profile {
    username: String,
    email: String,
}

pub async fn profile(
    services: State<Arc<Services>>,
    token: TokenHeader,
) -> Result<Json<Profile>, Response> {
    let claims = services
        .token_service
        .verify_access_token(token.to_bearer_token()?)
        .map_err(IntoResponse::into_response)?;

    let user = services
        .user_service
        .get_by_id(claims.sub)
        .await
        .map_err(IntoResponse::into_response)?
        .ok_or((StatusCode::UNAUTHORIZED, "user not found").into_response())?;

    Ok(Json(Profile {
        username: user.username,
        email: user.email,
    }))
}

#[derive(Deserialize)]
pub struct ActivateForm {
    email: String,
}

pub async fn send_activation_email(
    services: State<Arc<Services>>,
    Json(req): Json<ActivateForm>,
) -> Result<(), Response> {
    if !services
        .rate_limit_service
        .check_rate_limit(
            &format!("activation_email:{}", req.email),
            chrono::Duration::minutes(1),
        )
        .await
        .map_err(IntoResponse::into_response)?
    {
        return Err((StatusCode::TOO_MANY_REQUESTS, "").into_response());
    };

    let user = services
        .user_service
        .get_by_email(&req.email)
        .await
        .map_err(IntoResponse::into_response)?;

    let user = match user {
        Some(user) => user,
        None => return Ok(()),
    };

    if user.activated_at.is_some() {
        return Ok(());
    }

    generate_and_send_activation_email(services, user)?;

    Ok(())
}

fn generate_and_send_activation_email(
    services: State<Arc<Services>>,
    user: User,
) -> Result<(), Response> {
    let token = services
        .token_service
        .create_activation_code(user.id)
        .map_err(IntoResponse::into_response)?;

    services
        .email_service
        .send_activation_email(user.username, &user.email, &token)
        .map_err(IntoResponse::into_response)?;

    Ok(())
}

#[derive(Deserialize)]
pub struct ActivateQuery {
    code: String,
}

pub async fn activate(
    services: State<Arc<Services>>,
    Json(query): Json<ActivateQuery>,
) -> Result<(), Response> {
    let claims = services
        .token_service
        .verify_activation_code(&query.code)
        .map_err(IntoResponse::into_response)?;

    services
        .user_service
        .activate(claims.sub)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(())
}
