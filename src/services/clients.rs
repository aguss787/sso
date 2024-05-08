use crate::db::DbPool;
use crate::helpers::InternalError;
use std::sync::Arc;

pub use models::Client;

pub struct ClientService {
    pool: Arc<DbPool>,
}

impl ClientService {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl ClientService {
    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Option<Client>, InternalError> {
        let mut conn = self.pool.get().await?;
        Client::find_by_client_id(client_id, &mut conn)
            .await
            .map_err(Into::into)
    }
}

mod models {
    use crate::db::schema::clients;
    use argon2::PasswordVerifier;
    use diesel::{
        ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable, SelectableHelper,
    };
    use diesel_async::{AsyncPgConnection, RunQueryDsl};

    #[derive(Debug, Selectable, Queryable)]
    pub struct Client {
        client_secret: String,
        pub redirect_uri: String,
    }

    impl Client {
        pub fn is_secret_match(&self, secret: &str) -> Result<bool, argon2::password_hash::Error> {
            let argon2 = argon2::Argon2::default();
            let parsed_hash = argon2::PasswordHash::new(&self.client_secret)?;

            match argon2.verify_password(secret.as_bytes(), &parsed_hash) {
                Ok(_) => Ok(true),
                Err(error) => {
                    tracing::debug!(
                        error = error.to_string(),
                        "client secret verification failed"
                    );
                    Ok(false)
                }
            }
        }
    }

    impl Client {
        pub async fn find_by_client_id(
            client_id: &str,
            conn: &mut AsyncPgConnection,
        ) -> Result<Option<Self>, diesel::result::Error> {
            clients::table
                .select(Self::as_select())
                .filter(clients::client_id.eq(client_id))
                .first(conn)
                .await
                .optional()
                .map_err(Into::into)
        }
    }
}
