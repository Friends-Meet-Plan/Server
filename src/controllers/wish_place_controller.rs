use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::controllers::models::wish_place::{
    CreateWishPlaceBody, UpdateWishPlaceBody, VisitWishPlaceBody, WishPlaceQuery,
    WishPlaceResponse, WishPlaceStatusDto,
};
use crate::entities::friendship::{self, FriendshipStatus};
use crate::entities::wish_place::{self, WishPlaceStatus};
use crate::entities::{
    Event, Friendship, WishPlace, WishPlaceActiveModel, WishPlaceColumn,
};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/wish-places", get(get_wish_places).post(create_wish_place))
        .route("/wish-places/{id}", patch(update_wish_place).delete(delete_wish_place))
        .route("/wish-places/{id}/visit", post(visit_wish_place))
}

#[utoipa::path(
    get,
    path = "/wish-places",
    summary = "Список wish places пользователя",
    description = "Возвращает места из wish list пользователя `user_id`. Доступ: сам пользователь или принятый друг.",
    params(WishPlaceQuery),
    responses(
        (status = 200, description = "Список мест", body = [WishPlaceResponse]),
        (status = 401, description = "Не авторизован"),
        (status = 403, description = "Нет доступа")
    ),
    security(("bearer_auth" = [])),
    tag = "WishPlaces"
)]
pub async fn get_wish_places(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Query(query): Query<WishPlaceQuery>,
) -> Result<Json<Vec<WishPlaceResponse>>, (StatusCode, String)> {
    let me = auth.user_id;
    if me != query.user_id && !are_users_accepted_friends(&db, me, query.user_id).await? {
        return Err((StatusCode::FORBIDDEN, "access denied".to_string()));
    }

    let rows = WishPlace::find()
        .filter(WishPlaceColumn::UserId.eq(query.user_id))
        .order_by_desc(WishPlaceColumn::CreatedAt)
        .all(&db)
        .await
        .map_err(internal_error)?;

    Ok(Json(rows.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    post,
    path = "/wish-places",
    summary = "Создать wish place",
    description = "Создает место в wish list текущего пользователя.",
    request_body = CreateWishPlaceBody,
    responses(
        (status = 201, description = "Место создано", body = WishPlaceResponse),
        (status = 400, description = "Некорректные данные"),
        (status = 401, description = "Не авторизован")
    ),
    security(("bearer_auth" = [])),
    tag = "WishPlaces"
)]
pub async fn create_wish_place(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Json(body): Json<CreateWishPlaceBody>,
) -> Result<(StatusCode, Json<WishPlaceResponse>), (StatusCode, String)> {
    if body.title.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title is required".to_string()));
    }

    let model = WishPlaceActiveModel {
        user_id: Set(auth.user_id),
        title: Set(body.title),
        description: Set(body.description),
        location: Set(body.location),
        link: Set(body.link),
        status: Set(WishPlaceStatus::Active),
        visited_event_id: Set(None),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(to_response(model))))
}

#[utoipa::path(
    patch,
    path = "/wish-places/{id}",
    summary = "Обновить wish place",
    description = "Обновляет поля места в wish list. Только владелец.",
    request_body = UpdateWishPlaceBody,
    params(("id" = Uuid, Path, description = "ID места")),
    responses(
        (status = 200, description = "Место обновлено", body = WishPlaceResponse),
        (status = 400, description = "Некорректные данные"),
        (status = 401, description = "Не авторизован"),
        (status = 404, description = "Место не найдено")
    ),
    security(("bearer_auth" = [])),
    tag = "WishPlaces"
)]
pub async fn update_wish_place(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWishPlaceBody>,
) -> Result<Json<WishPlaceResponse>, (StatusCode, String)> {
    let row = WishPlace::find_by_id(id)
        .filter(WishPlaceColumn::UserId.eq(auth.user_id))
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "wish place not found".to_string()))?;

    if body.title.is_none()
        && body.description.is_none()
        && body.location.is_none()
        && body.link.is_none()
        && body.status.is_none()
    {
        return Err((StatusCode::BAD_REQUEST, "nothing to update".to_string()));
    }

    let mut active = row.into_active_model();

    if let Some(title) = body.title {
        if title.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "title cannot be empty".to_string()));
        }
        active.title = Set(title);
    }
    if let Some(description) = body.description {
        active.description = Set(Some(description));
    }
    if let Some(location) = body.location {
        active.location = Set(Some(location));
    }
    if let Some(link) = body.link {
        active.link = Set(Some(link));
    }
    if let Some(status) = body.status {
        active.status = Set(map_status(status));
        if !matches!(status, WishPlaceStatusDto::Visited) {
            active.visited_event_id = Set(None);
        }
    }

    let updated = active.update(&db).await.map_err(internal_error)?;
    Ok(Json(to_response(updated)))
}

#[utoipa::path(
    post,
    path = "/wish-places/{id}/visit",
    summary = "Отметить wish place как visited",
    description = "Отмечает место как visited и привязывает к событию `event_id`. Только владелец. Пользователь должен быть creator события.",
    request_body = VisitWishPlaceBody,
    params(("id" = Uuid, Path, description = "ID места")),
    responses(
        (status = 200, description = "Место отмечено как visited", body = WishPlaceResponse),
        (status = 401, description = "Не авторизован"),
        (status = 404, description = "Место или событие не найдено"),
        (status = 403, description = "Только creator события может отметить place как visited")
    ),
    security(("bearer_auth" = [])),
    tag = "WishPlaces"
)]
pub async fn visit_wish_place(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(body): Json<VisitWishPlaceBody>,
) -> Result<Json<WishPlaceResponse>, (StatusCode, String)> {
    let row = WishPlace::find_by_id(id)
        .filter(WishPlaceColumn::UserId.eq(auth.user_id))
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "wish place not found".to_string()))?;

    let event = Event::find_by_id(body.event_id)
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "event not found".to_string()))?;

    if event.creator_id != auth.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "only event creator can mark wish place as visited".to_string(),
        ));
    }

    let mut active = row.into_active_model();
    active.status = Set(WishPlaceStatus::Visited);
    active.visited_event_id = Set(Some(body.event_id));

    let updated = active.update(&db).await.map_err(internal_error)?;
    Ok(Json(to_response(updated)))
}

#[utoipa::path(
    delete,
    path = "/wish-places/{id}",
    summary = "Архивировать wish place",
    description = "Архивирует место (status=archived). Только владелец.",
    params(("id" = Uuid, Path, description = "ID места")),
    responses(
        (status = 204, description = "Место архивировано"),
        (status = 401, description = "Не авторизован"),
        (status = 404, description = "Место не найдено")
    ),
    security(("bearer_auth" = [])),
    tag = "WishPlaces"
)]
pub async fn delete_wish_place(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let row = WishPlace::find_by_id(id)
        .filter(WishPlaceColumn::UserId.eq(auth.user_id))
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "wish place not found".to_string()))?;

    let mut active = row.into_active_model();
    active.status = Set(WishPlaceStatus::Archived);
    active.update(&db).await.map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

fn to_response(model: wish_place::Model) -> WishPlaceResponse {
    WishPlaceResponse {
        id: model.id,
        user_id: model.user_id,
        title: model.title,
        description: model.description,
        location: model.location,
        link: model.link,
        status: model.status.to_string(),
        visited_event_id: model.visited_event_id,
        created_at: model.created_at.to_rfc3339(),
    }
}

fn map_status(value: WishPlaceStatusDto) -> WishPlaceStatus {
    match value {
        WishPlaceStatusDto::Active => WishPlaceStatus::Active,
        WishPlaceStatusDto::Visited => WishPlaceStatus::Visited,
        WishPlaceStatusDto::Archived => WishPlaceStatus::Archived,
    }
}

async fn are_users_accepted_friends(
    db: &DatabaseConnection,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<bool, (StatusCode, String)> {
    let row = Friendship::find()
        .filter(friendship::Column::Status.eq(FriendshipStatus::Accepted))
        .filter(
            Condition::any()
                .add(
                    Condition::all()
                        .add(friendship::Column::UserId.eq(user_a))
                        .add(friendship::Column::FriendId.eq(user_b)),
                )
                .add(
                    Condition::all()
                        .add(friendship::Column::UserId.eq(user_b))
                        .add(friendship::Column::FriendId.eq(user_a)),
                ),
        )
        .one(db)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
