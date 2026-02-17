use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::entities::{User, UserActiveModel, UserColumn, user};
use crate::controllers::models::{UpdateUserRequestBody, UserResponse, UserNameSearchQuery};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/users/me", get(get_me).patch(update_me))
        .route("/users/{id}", get(get_user_by_id))
        .route("/users/search", get(search_users))
}

/*
    Достаем auth: AuthUser из JWT
*/
async fn get_me(
    auth_user: AuthUser,
    State(db_connection): State<DatabaseConnection>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|err| (StatusCode::UNAUTHORIZED, "invalid user id".to_string()))?;

    let model = User::find_by_id(user_id)
        .one(&db_connection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(model) = model else {
        return Err((StatusCode::NOT_FOUND, "user not found".to_string()));
    };

    Ok(Json(UserResponse {
        id: model.id,
        username: model.username,
        avatar_url: model.avatar_url,
        bio: model.bio,
    }))
}

/*
    Достаем auth: AuthUser из JWT
    Json(payload): тело запроса
*/
async fn update_me(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateUserRequestBody>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&auth.user_id)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid user id".to_string()))?;

    let model = User::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(model) = model else {
        return Err((StatusCode::NOT_FOUND, "user not found".to_string()));
    };

    let mut active: UserActiveModel = model.into();

    if let Some(username) = payload.username {
        active.username = Set(username);
    }
    if let Some(avatar_url) = payload.avatar_url {
        active.avatar_url = Set(Some(avatar_url));
    }
    if let Some(bio) = payload.bio {
        active.bio = Set(Some(bio));
    }

    let model = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserResponse {
        id: model.id,
        username: model.username,
        avatar_url: model.avatar_url,
        bio: model.bio,
    }))
}

async fn get_user_by_id(
    State(db): State<DatabaseConnection>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let model = User::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(model) = model else {
        return Err((StatusCode::NOT_FOUND, "user not found".to_string()));
    };

    Ok(Json(UserResponse {
        id: model.id,
        username: model.username,
        avatar_url: model.avatar_url,
        bio: model.bio,
    }))
}

/*
    Query(query): Query<UserSearchQueryParameters> - query параметры
*/
async fn search_users(
    State(db): State<DatabaseConnection>,
    Query(query): Query<UserNameSearchQuery>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let username = query.username.unwrap_or_default();
    if username.trim().is_empty() {
        // TODO: возможно просто вернуть пустой vec
        return Err((
            StatusCode::BAD_REQUEST,
            "username query required".to_string(),
        ));
    }

    let models = User::find()
        .filter(UserColumn::Username.starts_with(username))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let users = models
        .into_iter()
        .map(|model| UserResponse {
            id: model.id,
            username: model.username,
            avatar_url: model.avatar_url,
            bio: model.bio,
        })
        .collect();

    Ok(Json(users))
}
