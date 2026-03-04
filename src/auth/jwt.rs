use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

const JWT_ISSUER: &str = "friends-server";
const JWT_AUDIENCE: &str = "friends-api";

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}

pub fn create_jwt(user_id: Uuid) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or("failed to calculate jwt expiration")?
        .timestamp() as usize;

    let payload = Payload {
        sub: user_id.to_string(),
        exp: expiration,
        iss: JWT_ISSUER.to_string(),
        aud: JWT_AUDIENCE.to_string(),
    };

    let secret = jwt_secret()?;
    encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| e.to_string())
}

pub fn verify_jwt(token: &str) -> Result<Payload, String> {
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
