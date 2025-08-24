// src/auth.rs

use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    data: HashMap<String, String>,
}

/// Creates a new JWT.
pub fn auth_jwt_create(user_id: &str, expires_hours: u64, data: HashMap<String, String>) -> Result<String, String> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expires_hours as i64))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        data,
    };

    let secret = crate::context::get_var("JWT_SECRET");
    if secret.is_empty() {
        return Err("JWT_SECRET is not set".to_string());
    }

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|e| e.to_string())
}

/// Verifies a JWT and returns the claims.
pub fn auth_jwt_verify(token: &str) -> Result<HashMap<String, String>, String> {
    let secret = crate::context::get_var("JWT_SECRET");
    if secret.is_empty() {
        return Err("JWT_SECRET is not set".to_string());
    }

    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::new(Algorithm::HS256))
        .map(|data| data.claims.data)
        .map_err(|e| e.to_string())
}

/// Hashes a password using bcrypt.
pub fn auth_hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())
}

/// Verifies a password against a bcrypt hash.
pub fn auth_verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}
