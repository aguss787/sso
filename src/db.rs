use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::{Pool, PoolError};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use std::error::Error;

pub mod schema;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbPoolError = PoolError;

pub fn database_pool(database_url: &str) -> Result<DbPool, Box<dyn Error>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(manager).build().map_err(Into::into)
}
