use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::controllers::models::calendar::{
    BusydayResponse, CalendarQuery, CalendarResponse, IsBusyRequest, PendingInviteResponse,
};
use crate::entities::friendship::{self, FriendshipStatus};
use crate::entities::user_event::{UserEventResponse, UserEventRole};
use crate::entities::{Busyday, BusydayColumn, Event, EventColumn, Friendship, UserEvent, UserEventColumn};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/calendar/is_busy", post(is_busy))
        .route("/users/me/calendar", get(get_my_calendar))
        .route("/users/{user_id}/calendar", get(get_user_calendar))
}

#[utoipa::path(
    post,
    path = "/calendar/is_busy",
    summary = "Проверить занятость дня",
    description = "Проверяет, занят ли день `date` у пользователя `id`. Доступ: сам пользователь или принятый друг.",
    request_body = IsBusyRequest,
    responses(
        (status = 200, description = "Результат проверки", body = bool),
        (status = 400, description = "Некорректная дата"),
        (status = 401, description = "Не авторизован"),
        (status = 403, description = "Нет доступа")
    ),
    security(("bearer_auth" = [])),
    tag = "Calendar"
)]
pub async fn is_busy(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<IsBusyRequest>,
) -> Result<Json<bool>, (StatusCode, String)> {
    let me = parse_auth(auth)?;
    if me != payload.id && !are_users_accepted_friends(&db, me, payload.id).await? {
        return Err((StatusCode::FORBIDDEN, "access denied".to_string()));
    }

    let date = parse_one_date(&payload.date)?;
    let busy = Busyday::find()
        .filter(BusydayColumn::UserId.eq(payload.id))
        .filter(BusydayColumn::Date.eq(date))
        .one(&db)
        .await
        .map_err(internal_error)?
        .is_some();

    Ok(Json(busy))
}

#[utoipa::path(
    get,
    path = "/users/me/calendar",
    summary = "Мой календарь",
    description = "Возвращает календарь текущего пользователя за диапазон from..to: busy_days, pending_invites и past_events.",
    params(CalendarQuery),
    responses(
        (status = 200, description = "Данные календаря", body = CalendarResponse),
        (status = 400, description = "Некорректный диапазон дат"),
        (status = 401, description = "Не авторизован")
    ),
    security(("bearer_auth" = [])),
    tag = "Calendar"
)]
pub async fn get_my_calendar(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, (StatusCode, String)> {
    let me = parse_auth(auth)?;
    Ok(Json(build_calendar_response(&db, me, &query).await?))
}

#[utoipa::path(
    get,
    path = "/users/{user_id}/calendar",
    summary = "Календарь пользователя",
    description = "Возвращает календарь пользователя за диапазон from..to: busy_days, pending_invites и past_events. Доступ: сам пользователь или принятый друг.",
    params(
        ("user_id" = Uuid, Path, description = "ID пользователя"),
        CalendarQuery
    ),
    responses(
        (status = 200, description = "Данные календаря", body = CalendarResponse),
        (status = 400, description = "Некорректный диапазон дат"),
        (status = 401, description = "Не авторизован"),
        (status = 403, description = "Нет доступа")
    ),
    security(("bearer_auth" = [])),
    tag = "Calendar"
)]
pub async fn get_user_calendar(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, (StatusCode, String)> {
    let me = parse_auth(auth)?;
    if me != user_id && !are_users_accepted_friends(&db, me, user_id).await? {
        return Err((StatusCode::FORBIDDEN, "access denied".to_string()));
    }

    Ok(Json(build_calendar_response(&db, user_id, &query).await?))
}

async fn build_calendar_response(
    db: &DatabaseConnection,
    user_id: Uuid,
    query: &CalendarQuery,
) -> Result<CalendarResponse, (StatusCode, String)> {
    let (from, to) = parse_date_range(&query.from, &query.to)?;

    let busy_rows = Busyday::find()
        .filter(BusydayColumn::UserId.eq(user_id))
        .filter(BusydayColumn::Date.gte(from))
        .filter(BusydayColumn::Date.lte(to))
        .order_by_asc(BusydayColumn::Date)
        .all(db)
        .await
        .map_err(internal_error)?;

    let busy_days = busy_rows
        .iter()
        .map(|row| BusydayResponse {
            id: row.id,
            user_id: row.user_id,
            date: row.date.to_string(),
            event_id: row.event_id,
        })
        .collect::<Vec<_>>();

    let today = Utc::now().date_naive();
    let past_events = busy_rows
        .iter()
        .filter(|row| row.date < today)
        .map(|row| row.date.to_string())
        .collect::<Vec<_>>();

    let pending_rows = UserEvent::find()
        .filter(UserEventColumn::UserId.eq(user_id))
        .filter(UserEventColumn::Role.eq(UserEventRole::Participant))
        .filter(UserEventColumn::ResponseStatus.eq(UserEventResponse::Pending))
        .all(db)
        .await
        .map_err(internal_error)?;

    let event_ids = pending_rows
        .into_iter()
        .map(|row| row.event_id)
        .collect::<Vec<_>>();

    let mut pending_invites = Vec::new();
    if !event_ids.is_empty() {
        let events = Event::find()
            .filter(EventColumn::Id.is_in(event_ids))
            .filter(EventColumn::Date.gte(from))
            .filter(EventColumn::Date.lte(to))
            .order_by_asc(EventColumn::Date)
            .all(db)
            .await
            .map_err(internal_error)?;

        let mut by_date = HashMap::new();
        for event in events {
            by_date.insert(event.id, event.date.to_string());
        }

        pending_invites = by_date
            .into_iter()
            .map(|(event_id, date)| PendingInviteResponse { event_id, date })
            .collect::<Vec<_>>();
        pending_invites.sort_by(|a, b| a.date.cmp(&b.date));
    }

    Ok(CalendarResponse {
        from: from.to_string(),
        to: to.to_string(),
        busy_days,
        pending_invites,
        past_events,
    })
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

fn parse_auth(auth: AuthUser) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(&auth.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid user id".to_string()))
}

fn parse_date_range(from: &str, to: &str) -> Result<(NaiveDate, NaiveDate), (StatusCode, String)> {
    let from_date = parse_one_date(from)?;
    let to_date = parse_one_date(to)?;

    if from_date > to_date {
        return Err((StatusCode::BAD_REQUEST, "`from` must be <= `to`".to_string()));
    }

    Ok((from_date, to_date))
}

fn parse_one_date(value: &str) -> Result<NaiveDate, (StatusCode, String)> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid date format `{value}`, expected YYYY-MM-DD"),
        )
    })
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
