use diesel::pg::Pg;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

fn main() {
    dotenv::from_filename(".env.local").ok();

    let url = env::var("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let mut connection: diesel::PgConnection = diesel::connection::Connection::establish(&url)
        .expect("Failed to establish a database connection");

    run_migrations(&mut connection).unwrap()
}
