use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = argon2::Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(error) => {
            tracing::debug!(error = error.to_string(), "password verification failed");
            Ok(false)
        }
    }
}
