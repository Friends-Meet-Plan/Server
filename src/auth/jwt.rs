use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use uuid::Uuid;

const JWT_ISSUER: &str = "friends-server";
const JWT_AUDIENCE: &str = "friends-api";

enum TokenType {
    Refresh,
    Access,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Refresh => f.write_str("refresh"),
            TokenType::Access => f.write_str("access"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
    pub token_type: String,
}

pub fn create_access_jwt(user_id: Uuid) -> Result<String, String> {
    create_jwt(user_id, TokenType::Access, Duration::minutes(15))
}

pub fn create_refresh_jwt(user_id: Uuid) -> Result<String, String> {
    create_jwt(user_id, TokenType::Refresh, Duration::days(30))
}

fn create_jwt(
    user_id: Uuid,
    token_type: TokenType,
    ttl: Duration
) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(ttl)
        .ok_or("failed to calculate jwt expiration")?
        .timestamp() as usize;

    let payload = Payload {
        sub: user_id.to_string(),
        exp: expiration,
        iss: JWT_ISSUER.to_string(),
        aud: JWT_AUDIENCE.to_string(),
        token_type: token_type.to_string(),
    };

    let secret = jwt_secret()?;
    encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| e.to_string())
}

pub fn verify_access_jwt(token: &str) -> Result<Payload, String> {
    let payload = verify_jwt(token)?;
    if payload.token_type != TokenType::Access.to_string() {
        return Err("invalid token type for protected endpoint".to_string());
    }
    Ok(payload)
}

pub fn verify_refresh_jwt(token: &str) -> Result<Payload, String> {
    let payload = verify_jwt(token)?;
    if payload.token_type != TokenType::Refresh.to_string() {
        return Err("invalid token type for refresh endpoint".to_string());
    }
    Ok(payload)
}

fn verify_jwt(token: &str) -> Result<Payload, String> {
    let secret = jwt_secret()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[JWT_ISSUER]);
    validation.set_audience(&[JWT_AUDIENCE]);

    decode::<Payload>(
        token,
        &DecodingKey::from_secret(&secret),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}

fn jwt_secret() -> Result<Vec<u8>, String> {
    env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .map_err(|_| "JWT_SECRET must be set".to_string())
}
