use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::auth::jwt::{create_access_jwt, create_refresh_jwt, verify_refresh_jwt};
use crate::controllers::models::user_response::UserResponse;
use crate::controllers::models::{
    AuthRequestBody, LoginRequestBody, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
};
use crate::entities::{
    RefreshToken, RefreshTokenActiveModel, RefreshTokenColumn, User, UserActiveModel, user,
};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = AuthRequestBody,
    responses(
        (status = 201, description = "User created")
    )
)]
pub async fn register(
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
    let _ = active.insert(&db_connection).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("duplicate key") || msg.contains("unique") {
            (StatusCode::CONFLICT, "username already exists".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequestBody,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
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

    let parsed_hash = PasswordHash::new(&model.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid password hash".to_string(),
        )
    })?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))?;

    let access_token = create_access_jwt(model.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let refresh_issue = create_refresh_jwt(model.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    persist_refresh_token(
        &db_connection,
        refresh_issue.jti,
        model.id,
        refresh_issue.expires_at,
    )
    .await?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: refresh_issue.token,
        user: UserResponse {
            id: model.id,
            username: model.username,
            avatar_url: model.avatar_url,
            bio: model.bio,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "New token pair", body = RefreshTokenResponse),
        (status = 401, description = "Invalid refresh token")
    )
)]
pub async fn refresh(
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, String)> {
    let payload = verify_refresh_jwt(&body.refresh_token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        )
    })?;

    let user_id = Uuid::parse_str(&payload.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        )
    })?;

    let jti = payload
        .jti
        .as_ref()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        ))
        .and_then(|v| {
            Uuid::parse_str(v).map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "invalid refresh token".to_string(),
                )
            })
        })?;

    let user_exists = User::find_by_id(user_id)
        .one(&db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some();

    if !user_exists {
        return Err((
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        ));
    }

    let active_token = RefreshToken::find_by_id(jti)
        .filter(RefreshTokenColumn::UserId.eq(user_id))
        .one(&db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        ))?;

    if active_token.revoked_at.is_some() || active_token.expires_at < Utc::now() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        ));
    }

    revoke_refresh_token(&db_connection, jti).await?;

    let access_token = create_access_jwt(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_issue = create_refresh_jwt(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    persist_refresh_token(
        &db_connection,
        refresh_issue.jti,
        user_id,
        refresh_issue.expires_at,
    )
    .await?;

    Ok(Json(RefreshTokenResponse {
        access_token,
        refresh_token: refresh_issue.token,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 204, description = "Refresh token revoked"),
        (status = 401, description = "Invalid refresh token")
    )
)]
pub async fn logout(
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let payload = verify_refresh_jwt(&body.refresh_token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        )
    })?;
    let jti = payload
        .jti
        .as_ref()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "invalid refresh token".to_string(),
        ))
        .and_then(|v| {
            Uuid::parse_str(v).map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "invalid refresh token".to_string(),
                )
            })
        })?;

    revoke_refresh_token(&db_connection, jti).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn persist_refresh_token(
    db_connection: &DatabaseConnection,
    token_id: Uuid,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<(), (StatusCode, String)> {
    let active = RefreshTokenActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        expires_at: Set(expires_at.into()),
        revoked_at: Set(None),
        ..Default::default()
    };
    active
        .insert(db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

async fn revoke_refresh_token(
    db_connection: &DatabaseConnection,
    token_id: Uuid,
) -> Result<(), (StatusCode, String)> {
    if let Some(row) = RefreshToken::find_by_id(token_id)
        .one(db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let mut active: RefreshTokenActiveModel = row.into();
        active.revoked_at = Set(Some(Utc::now().into()));
        active
            .update(db_connection)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(())
}
