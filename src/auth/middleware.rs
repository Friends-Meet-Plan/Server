use crate::auth::jwt::verify_jwt;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

pub struct AuthUser {
    pub user_id: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // let auth_header = parts
        //     .headers
        //     .get("Authorization")
        //     .and_then(|value| value.to_str().ok());
        // if let Some(header) = auth_header {
        //     if header.starts_with("Bearer ") {
        //         let token = header.trim_start_matches("Bearer ");
        //         if let Some(payload) = verify_jwt(token) {
        //             return Ok(AuthUser {
        //                 user_id: payload.sub,
        //             });
        //         }
        //     }
        // }
        //
        // Err((StatusCode::UNAUTHORIZED, "Missing or invalid Bearer token"))
        // TODO: включить в release
        return Ok(AuthUser { user_id: "fd94d39f-c469-4f52-8d6d-34f6b7c0546c".to_string() });
    }
}
