
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        errors::Error as Argon2Error,
    },
    Argon2
};

use crate::Error;

pub(crate) fn hash_password(password: &str) -> Result<String, Argon2Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub(crate) fn verify_password(hash: &str, password: &str) -> Result<bool, Argon2Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    let result = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
    Ok(result)
}

pub trait UserStore {
    fn check_password(&self, username: &str, password: &str) -> Result<bool, Error>;

    /// Update the password for a user. If the user does not exist, it will be created.
    fn set_password(&self, username: &str, password: &str) -> Result<(), Error>;

    fn delete_user(&self, username: &str) -> Result<(), Error>;

    fn generate_api_key(&self, username: &str) -> Result<String, Error>;

    fn get_user_from_api_key(&self, api_key: &str) -> Result<Option<String>, Error>;

    fn list_users(&self) -> Result<Vec<String>, Error>;
}
