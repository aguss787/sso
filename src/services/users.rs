mod password;

use crate::db::DbPool;
use crate::helpers::{InternalError, ManualErrorHandle, ManualErrorHandling};
use crate::services::users::password::{hash_password, verify_password};
use axum::response::{IntoResponse, Response};
use std::ops::Deref;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub use models::User;

pub struct UserService {
    db_pool: Arc<DbPool>,
}

impl UserService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }
}

impl UserService {
    #[instrument(skip(self))]
    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User, RegisterError> {
        let mut conn = self.db_pool.get().await?;
        models::NewUser::new(username, email, hash_password(&password))
            .save(&mut conn)
            .await
            .manual_error_handling()
            .map_err(Into::into)
    }

    pub async fn validate_and_return(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, UserValidationError> {
        let mut conn = self.db_pool.get().await?;
        let user = User::find_by_username(username, &mut conn).await?;

        match user {
            None => {
                tracing::info!(user.username = username, "user not found");
                Err(UserValidationError::UserNotFound)
            }
            Some(user) if !verify_password(password, &user.password)? => {
                tracing::info!(
                    user.id = user.id.to_string(),
                    user.username,
                    "invalid password"
                );
                Err(UserValidationError::InvalidPassword)
            }
            Some(user) if user.activated_at.is_none() => {
                tracing::info!(
                    user.id = user.id.to_string(),
                    user.username,
                    "user not activated"
                );
                Err(UserValidationError::NotActivated)
            }
            Some(user) => {
                tracing::info!(
                    user.id = user.id.to_string(),
                    user.username,
                    "user validated"
                );
                Ok(user)
            }
        }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<User>, InternalError> {
        let mut conn = self.db_pool.get().await?;

        User::find_by_id(id, &mut conn).await.map_err(Into::into)
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, InternalError> {
        let mut conn = self.db_pool.get().await?;

        User::find_by_email(email, &mut conn)
            .await
            .map_err(Into::into)
    }

    pub async fn activate(&self, id: Uuid) -> Result<(), InternalError> {
        let mut conn = self.db_pool.get().await?;

        User::activate(id, &mut conn).await.map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("username already taken")]
    UsernameTaken,
    #[error("email already taken")]
    EmailTaken,
    #[error("internal error: {0}")]
    InternalError(InternalError),
}

impl<T: Into<InternalError>> From<T> for RegisterError {
    fn from(error: T) -> Self {
        Self::InternalError(error.into())
    }
}

impl From<ManualErrorHandling<diesel::result::Error>> for RegisterError {
    fn from(error: ManualErrorHandling<diesel::result::Error>) -> Self {
        match error.deref() {
            diesel::result::Error::DatabaseError(_, ref info) => match info.constraint_name() {
                Some("unique_username") => {
                    tracing::info!("username taken");
                    Self::UsernameTaken
                }
                Some("unique_email") => {
                    tracing::info!("email taken");
                    Self::EmailTaken
                }
                _ => {
                    tracing::error!(
                        error = error.to_string(),
                        constraint_name = info.constraint_name(),
                        "unhandled database error"
                    );
                    Self::InternalError(error.into_inner().into())
                }
            },
            _ => Self::InternalError(error.into_inner().into()),
        }
    }
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        match self {
            Self::UsernameTaken => {
                (axum::http::StatusCode::CONFLICT, "username already taken").into_response()
            }
            Self::EmailTaken => {
                (axum::http::StatusCode::CONFLICT, "email already taken").into_response()
            }
            Self::InternalError(e) => e.into_response(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserValidationError {
    #[error("username already taken")]
    UserNotFound,
    #[error("email already taken")]
    InvalidPassword,
    #[error("user not activated")]
    NotActivated,
    #[error("internal error: {0}")]
    InternalError(InternalError),
}

impl<T: Into<InternalError>> From<T> for UserValidationError {
    fn from(error: T) -> Self {
        Self::InternalError(error.into())
    }
}

mod models {
    use diesel::{
        BoolExpressionMethods, ExpressionMethods, Insertable, OptionalExtension, QueryDsl,
        Queryable, Selectable, SelectableHelper,
    };
    use diesel_async::{AsyncPgConnection, RunQueryDsl};
    use uuid::Uuid;

    use crate::db::schema::users;

    #[derive(Debug, Selectable, Queryable)]
    pub struct User {
        pub id: Uuid,
        pub username: String,
        pub email: String,
        pub password: String,
        pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl User {
        pub async fn find_by_id(
            id: Uuid,
            conn: &mut AsyncPgConnection,
        ) -> Result<Option<Self>, diesel::result::Error> {
            users::table
                .select(Self::as_select())
                .filter(users::id.eq(id))
                .first(conn)
                .await
                .optional()
        }

        pub async fn find_by_username(
            username: &str,
            conn: &mut AsyncPgConnection,
        ) -> Result<Option<Self>, diesel::result::Error> {
            users::table
                .select(Self::as_select())
                .filter(users::username.eq(username))
                .first(conn)
                .await
                .optional()
        }

        pub async fn find_by_email(
            email: &str,
            conn: &mut AsyncPgConnection,
        ) -> Result<Option<Self>, diesel::result::Error> {
            users::table
                .select(Self::as_select())
                .filter(users::email.eq(email))
                .first(conn)
                .await
                .optional()
        }

        pub async fn activate(
            id: Uuid,
            conn: &mut AsyncPgConnection,
        ) -> Result<(), diesel::result::Error> {
            diesel::update(users::table)
                .filter(users::id.eq(id).and(users::activated_at.is_null()))
                .set(users::activated_at.eq(Some(chrono::Utc::now())))
                .execute(conn)
                .await?;

            Ok(())
        }
    }

    #[derive(Debug, Insertable)]
    #[diesel(table_name = users)]
    pub struct NewUser {
        username: String,
        email: String,
        password: String,
    }

    impl NewUser {
        pub fn new(username: String, email: String, password: String) -> Self {
            Self {
                username,
                email,
                password,
            }
        }

        pub async fn save(
            self,
            conn: &mut AsyncPgConnection,
        ) -> Result<User, diesel::result::Error> {
            diesel::insert_into(users::table)
                .values(self)
                .returning(User::as_select())
                .load(conn)
                .await
                .map(|mut user| user.pop().expect("inserted user not returned"))
        }
    }
}
