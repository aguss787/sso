use crate::helpers::InternalError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};
use url::Url;

pub struct EmailService {
    base_url: Url,
    smtp_transport: SmtpTransport,
    sender_email: Address,
    sender_name: Option<String>,
}

impl EmailService {
    pub fn new(
        base_url: Url,
        host: &str,
        username: String,
        password: String,
        sender_email: Address,
    ) -> Self {
        Self {
            base_url,
            smtp_transport: SmtpTransport::starttls_relay(host)
                .unwrap()
                .credentials(Credentials::new(username, password))
                .build(),

            sender_email,
            sender_name: None,
        }
    }

    pub fn set_sender_name(mut self, name: String) -> Self {
        self.sender_name = Some(name);
        self
    }

    pub fn send_activation_email(
        &self,
        name: String,
        email: &str,
        token: &str,
    ) -> Result<(), ActivationEmailError> {
        let mut url = self.base_url.clone();
        url.query_pairs_mut().append_pair("code", token);

        let email = Message::builder()
            .from(Mailbox::new(
                self.sender_name.clone(),
                self.sender_email.clone(),
            ))
            .to(Mailbox::new(Some(name), email.parse()?))
            .subject("Activation Link for agus.dev SSO")
            .header(ContentType::TEXT_PLAIN)
            .body(url.to_string())?;

        self.smtp_transport.send(&email)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActivationEmailError {
    #[error("Invalid email address: {0}")]
    InvalidEmail(#[from] lettre::address::AddressError),
    #[error("Internal error: {0}")]
    InternalError(InternalError),
}

impl<T> From<T> for ActivationEmailError
where
    T: Into<InternalError>,
{
    fn from(error: T) -> Self {
        Self::InternalError(error.into())
    }
}

impl IntoResponse for ActivationEmailError {
    fn into_response(self) -> Response {
        match self {
            ActivationEmailError::InvalidEmail(_) => {
                (StatusCode::BAD_REQUEST, "invalid email address").into_response()
            }
            ActivationEmailError::InternalError(e) => e.into_response(),
        }
    }
}
