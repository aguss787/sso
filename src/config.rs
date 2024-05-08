use std::env;

pub struct Config {
    pub port: u16,
    pub postgres_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub base_url: String,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_sender_email: String,
    pub smtp_sender_name: String,
}

impl Config {
    pub fn read_env() -> Self {
        Config {
            port: env::var("SERVER_PORT")
                .unwrap_or("3001".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
            postgres_url: env::var("POSTGRES_URL").expect("POSTGRES_URL must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            base_url: env::var("BASE_URL").expect("BASE_URL must be set"),
            smtp_host: env::var("SMTP_HOST").expect("SMTP_HOST must be set"),
            smtp_username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
            smtp_sender_email: env::var("SMTP_SENDER_EMAIL")
                .expect("SMTP_SENDER_EMAIL must be set"),
            smtp_sender_name: env::var("SMTP_SENDER_NAME").expect("SMTP_SENDER_NAME must be set"),
        }
    }
}
