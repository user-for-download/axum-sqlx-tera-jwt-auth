use argon2::{Argon2, password_hash::{
    PasswordHash,
    PasswordHasher, PasswordVerifier, rand_core::OsRng, SaltString},
};
use argon2::password_hash::Error;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use thiserror::Error;

use crate::auth::models::Claims;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub async fn encode_jwt(email: String, purpose: String, duration: i64) -> Result<String, String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(duration)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claims {
        iat,
        exp,
        email,
        purpose,
    };

    encode(
        &Header::default(),
        &claim,
        &KEYS.encoding,
    ).map_err(|err| err.to_string())
}

#[derive(Debug, Error)]
pub enum DecodeTokenError {
    #[error("Missing JWT_SECRET environment variable")]
    MissingSecret,
    #[error("Failed to decode token: {0}")]
    DecodeError(String),
    #[error("Token is expired!")]
    ExpiredToken,
}

pub async fn decode_token(token: String) -> Result<Option<Claims>, DecodeTokenError> {
    let decoded = decode::<Claims>(
        &*token,
        &KEYS.decoding,
        &jsonwebtoken::Validation::default(),
    );

    match decoded {
        Ok(data) => {
            let claims = data.claims;
            let now = Utc::now().timestamp() as usize;
            if claims.exp > now {
                Ok(Some(claims))
            } else {
                Err(DecodeTokenError::ExpiredToken)
            }
        }
        Err(err) => Err(DecodeTokenError::DecodeError(err.to_string())),
    }
}

pub fn ar_hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(e),
    }
}

pub fn ar_verify_password(password: &str, hashed_password: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hashed_password)?;
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(e) => Err(e)
    }
}