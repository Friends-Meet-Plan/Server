use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Serialize;

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::auth::jwt::create_jwt;
use crate::controllers::models::auth_request_body::AuthRequestBody;
use crate::controllers::models::login_request_body::LoginRequestBody;
use crate::controllers::models::login_response::LoginResponse;
use crate::controllers::models::user_response::UserResponse;
use crate::entities::{User, UserActiveModel, user};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

async fn register(
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<AuthRequestBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    if body.username.trim().is_empty() || body.password.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "username and password required".to_string(),
        ));
    }
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hashing failed".to_string(),
            )
        })?
        .to_string();

    let active = UserActiveModel {
        username: Set(body.username),
        password_hash: Set(password_hash),
        ..Default::default()
    };
    let _model = active
        .insert(&db_connection)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique") {
                (StatusCode::CONFLICT, "username already exists".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        })?;

    Ok(StatusCode::CREATED)
}

async fn login(
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<LoginRequestBody>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    if body.username.trim().is_empty() || body.password.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "username and password required".to_string(),
        ));
    }

    let model = User::find()
        .filter(user::Column::Username.eq(body.username))
        .one(&db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(model) = model else {
        return Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_string()));
    };

    let parsed_hash = PasswordHash::new(&model.password_hash)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid password hash".to_string(),
            )
        })?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))?;

    let token = create_jwt(model.id.to_string());
    Ok(Json(LoginResponse {
        token,
        user: UserResponse {
            id: model.id,
            username: model.username,
            avatar_url: model.avatar_url,
            bio: model.bio,
        },
    }))
}
