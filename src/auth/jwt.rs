use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use std::env;

/// JWT: HEADER.PAYLOAD.SIGNATURE

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub sub: String, // например user id
    pub exp: usize
}

pub fn create_jwt(user_id: String) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let payload = Payload {
        sub: user_id,
        exp: expiration
    };

    /*
    Header::default:
    {
        "alg": "HS256",
        "typ": "JWT"
    }
    Enconding: создание ключа для подписи
    */
    return encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(&jwt_secret())
    )
    .unwrap();
}

pub fn verify_jwt(token: &str) -> Option<Payload> {
    // Decoding jwt token into Payload
    decode::<Payload>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default()
    )
    .map(|data| data.claims)
    .ok()
}

fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes()
}
