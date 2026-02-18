use std::collections::HashMap;
use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use crate::auth::middleware::AuthUser;
use crate::controllers::models::{FriendIdBody, UserDTO};
use crate::entities::{friendship, user, Friendship, User, UserColumn};
use crate::entities::friendship::FriendshipStatus;

pub use friendship::ActiveModel as FriendshipActive;
pub use friendship::Column as FriendshipColumn;

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/friends", get(get_friends))
        .route("/friends/request", post(friend_request))
        .route("/friends/incoming", get(get_incoming))
        .route("/friends/outgoing", get(get_outgoing))
}

async fn get_friends(
    auth: AuthUser,
    State(db_connection): State<DatabaseConnection>,
) -> Result<Json<Vec<UserDTO>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let rows = Friendship::find()
        .filter(friendship::Column::Status.eq(FriendshipStatus::Accepted))
        .filter(
            Condition::any()
                .add(FriendshipColumn::UserId.eq(me))
                .add(FriendshipColumn::FriendId.eq(me))
        )
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    let ids: Vec<Uuid> = rows
        .into_iter()
        .map(|row| { if row.friend_id == me { row.user_id } else { row.friend_id } })
        .collect();

    if ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let users = User::find()
        .filter(UserColumn::Id.is_in(ids))
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        users
            .into_iter()
            .map(|model| to_user_dto(&model))
            .collect()
    ))
}

async fn get_incoming(
    auth: AuthUser,
    State(db_connection): State<DatabaseConnection>,
) -> Result<Json<Vec<UserDTO>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;

    let rows = Friendship::find()
        .filter(FriendshipColumn::FriendId.eq(me))
        .filter(FriendshipColumn::Status.eq(FriendshipStatus::Pending))
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    let ids: Vec<Uuid> = rows
        .into_iter()
        .map(|row| row.user_id)
        .collect();
    if ids.is_empty() { return Ok(Json(vec![])); }

    let users = User::find()
        .filter(UserColumn::Id.is_in(ids))
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        users
            .into_iter()
            .map(|model| to_user_dto(&model))
            .collect()
    ))
}

async fn get_outgoing(
    auth: AuthUser,
    State(db_connection): State<DatabaseConnection>,
) -> Result<Json<Vec<UserDTO>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;

    let rows = Friendship::find()
        .filter(FriendshipColumn::UserId.eq(me))
        .filter(FriendshipColumn::Status.eq(FriendshipStatus::Pending))
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    let ids: Vec<Uuid> = rows
        .into_iter()
        .map(|row| row.friend_id)
        .collect();
    if ids.is_empty() { return Ok(Json(vec![])); }

    let users = User::find()
        .filter(UserColumn::Id.is_in(ids))
        .all(&db_connection)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        users
            .into_iter()
            .map(|model| to_user_dto(&model))
            .collect()
    ))
}

async fn friend_request(
    auth: AuthUser,
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<FriendIdBody>,
) -> Result<StatusCode, (StatusCode, String) > {
    let me = parse_auth_user_id(auth)?;
    if body.friend_id == me { return Err((StatusCode::BAD_REQUEST, "cannot add yourself".to_string())); }

    let frindship_active = FriendshipActive {
        user_id: Set(me),
        friend_id: Set(body.friend_id),
        status: Set(FriendshipStatus::Pending),
        ..Default::default()
    };

    frindship_active
        .insert(&db_connection)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::CREATED)
}

/*
## Friendships

- `POST /friends/requests/:id/accept`
  - принять входящий запрос (status=accepted)
- `POST /friends/requests/:id/decline`
  - отклонить входящий запрос (status=declined или удалить запись)
- `DELETE /friends/:id`
  - удалить из друзей (разорвать accepted связь)
 */



// MARK: Helper methods
fn parse_auth_user_id(auth: AuthUser) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(&auth.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn to_user_dto(u: &user::Model) -> UserDTO {
    UserDTO {
        id: u.id,
        username: u.username.clone(),
        avatar_url: u.avatar_url.clone(),
        bio: u.bio.clone(),
    }
}
