mod config;
mod db;
mod helpers;
mod kvs;
mod routes;
mod services;

use crate::config::Config;
use crate::db::database_pool;
use crate::kvs::kvs_pool;
use crate::services::clients::ClientService;
use crate::services::email::EmailService;
use crate::services::rate_limit::RateLimitService;
use crate::services::tokens::{JwtSecret, TokenService};
use crate::services::users::UserService;
use axum::routing::{get, post};
use axum::Router;
use services::oauth2::Oauth2Service;
use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::services::ServeFile;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;

struct Services {
    user_service: Arc<UserService>,
    client_service: Arc<ClientService>,
    token_service: Arc<TokenService>,
    oauth2_service: Arc<Oauth2Service>,
    email_service: Arc<EmailService>,
    rate_limit_service: Arc<RateLimitService>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::from_filename(".env").ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .init();

    let config = Config::read_env();

    let port = config.port;
    let db_pool = Arc::new(
        database_pool(&config.postgres_url).expect("Failed to create database connection pool"),
    );

    let kvs_pool =
        Arc::new(kvs_pool(&config.redis_url).expect("Failed to create KVS connection pool"));

    let user_service = Arc::new(UserService::new(db_pool.clone()));
    let client_service = Arc::new(ClientService::new(db_pool.clone()));
    let token_service = Arc::new(TokenService::new(
        kvs_pool.clone(),
        JwtSecret(config.jwt_secret.as_ref()),
    ));
    let email_service = Arc::new(
        EmailService::new(
            format!("{}/activate", config.base_url)
                .parse()
                .expect("failed to parse base URL"),
            &config.smtp_host,
            config.smtp_username.clone(),
            config.smtp_password.clone(),
            config
                .smtp_sender_email
                .parse()
                .expect("failed to parse sender email address"),
        )
        .set_sender_name(config.smtp_sender_name.clone()),
    );
    let rate_limit_service = Arc::new(RateLimitService::new(kvs_pool.clone()));
    let oauth2_service = Arc::new(Oauth2Service::new(
        token_service.clone(),
        client_service.clone(),
    ));

    let services = Arc::new(Services {
        user_service,
        client_service,
        oauth2_service,
        token_service,
        email_service,
        rate_limit_service,
    });

    let app = Router::new()
        .route(
            "/register",
            get(|req| ServeFile::new("static/register.html").oneshot(req)).post(routes::register),
        )
        .route(
            "/oauth2/login",
            get(|req| ServeFile::new("static/login.html").oneshot(req)).post(routes::login),
        )
        .route("/oauth2/token", post(routes::token))
        .route(
            "/activate",
            get(|req| ServeFile::new("static/activate.html").oneshot(req)).post(routes::activate),
        )
        .route("/send-activation", post(routes::send_activation_email))
        .route("/profile", get(routes::profile))
        .with_state(services)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

    tracing::info!("Listening on 0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
