use std::collections::{HashMap, HashSet};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::controllers::models::{
    AcceptInvitationRequest, BusydayResponse, CalendarQuery, CalendarResponse,
    CreateInvitationRequest, CreatedInvitationResponse, InvitationDateResponse,
    InvitationResponse, PendingInviteResponse,
};
use crate::entities::friendship::{self, FriendshipStatus};
use crate::entities::invitation::InvitationStatus;
use crate::entities::{
    Busyday, BusydayActiveModel, BusydayColumn, Friendship, Invitation, InvitationColumn,
    InvitationDate, InvitationDateActiveModel, InvitationDateColumn, invitation,
};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/invitations", post(create_invitation))
        .route("/invitations/incoming", get(get_incoming_invitations))
        .route("/invitations/outgoing", get(get_outgoing_invitations))
        .route("/invitations/{id}", get(get_invitation_by_id))
        .route("/invitations/{id}/accept", post(accept_invitation))
        .route("/invitations/{id}/decline", post(decline_invitation))
        .route("/invitations/{id}/cancel", post(cancel_invitation))
        .route("/users/{user_id}/calendar", get(get_user_calendar))
        .route("/users/me/busydays", get(get_my_busydays))
}

#[utoipa::path(
    post,
    path = "/invitations",
    summary = "Создать приглашение",
    description = "Создаёт новое приглашение от текущего пользователя к `to_user_id` со списком предложенных дат. Проверяет: нельзя приглашать самого себя, даты должны быть уникальными и не в прошлом, между пользователями должна быть принятая дружба. В ответе возвращает только `id` созданного приглашения.",
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitation created", body = CreatedInvitationResponse),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Users are not friends")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn create_invitation(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<CreatedInvitationResponse>), (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    validate_create_invitation_payload(me, &payload)?;

    if !are_users_accepted_friends(&db, me, payload.to_user_id).await? {
        return Err((StatusCode::FORBIDDEN, "users are not friends".to_string()));
    }

    let parsed_dates = parse_and_validate_dates(&payload.dates)?;
    let invitation = invitation::ActiveModel {
        from_user_id: Set(me),
        to_user_id: Set(payload.to_user_id),
        status: Set(InvitationStatus::Pending),
        selected_date: Set(None),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(internal_error)?;

    for day in parsed_dates {
        InvitationDateActiveModel {
            invitation_id: Set(invitation.id),
            date: Set(day),
            ..Default::default()
        }
        .insert(&db)
        .await
        .map_err(map_db_constraint_error)?;
    }

    Ok((
        StatusCode::CREATED,
        Json(CreatedInvitationResponse { id: invitation.id }),
    ))
}

#[utoipa::path(
    get,
    path = "/invitations/incoming",
    summary = "Список входящих приглашений",
    description = "Возвращает только входящие приглашения в статусе `pending`, где текущий пользователь является получателем (`to_user_id`). Каждый элемент содержит поля приглашения и предложенные даты.",
    responses(
        (status = 200, description = "Incoming invitations", body = [InvitationResponse]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn get_incoming_invitations(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<InvitationResponse>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let condition = Condition::all()
        .add(InvitationColumn::ToUserId.eq(me))
        .add(InvitationColumn::Status.eq(InvitationStatus::Pending));

    let invitations = Invitation::find()
        .filter(condition)
        .order_by_desc(InvitationColumn::CreatedAt)
        .all(&db)
        .await
        .map_err(internal_error)?;

    let mut response = Vec::with_capacity(invitations.len());
    for invitation in invitations {
        response.push(load_invitation_response(&db, invitation.id).await?);
    }

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/invitations/outgoing",
    summary = "Список исходящих приглашений",
    description = "Возвращает только исходящие приглашения в статусе `pending`, созданные текущим пользователем (`from_user_id`). Каждый элемент содержит поля приглашения и предложенные даты.",
    responses(
        (status = 200, description = "Outgoing invitations", body = [InvitationResponse]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn get_outgoing_invitations(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<InvitationResponse>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let condition = Condition::all()
        .add(InvitationColumn::FromUserId.eq(me))
        .add(InvitationColumn::Status.eq(InvitationStatus::Pending));

    let invitations = Invitation::find()
        .filter(condition)
        .order_by_desc(InvitationColumn::CreatedAt)
        .all(&db)
        .await
        .map_err(internal_error)?;

    let mut response = Vec::with_capacity(invitations.len());
    for invitation in invitations {
        response.push(load_invitation_response(&db, invitation.id).await?);
    }

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/invitations/{id}",
    summary = "Детали приглашения",
    description = "Возвращает одно приглашение по id вместе со всеми предложенными датами. Доступно только участникам приглашения (отправителю или получателю).",
    params(
        ("id" = Uuid, Path, description = "Invitation id")
    ),
    responses(
        (status = 200, description = "Invitation details", body = InvitationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invitation not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn get_invitation_by_id(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvitationResponse>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let invitation = Invitation::find_by_id(id)
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "invitation not found".to_string()))?;

    if invitation.from_user_id != me && invitation.to_user_id != me {
        return Err((StatusCode::FORBIDDEN, "forbidden".to_string()));
    }

    Ok(Json(load_invitation_response(&db, id).await?))
}

#[utoipa::path(
    post,
    path = "/invitations/{id}/accept",
    summary = "Принять приглашение",
    description = "Позволяет получателю принять приглашение в статусе `pending` и выбрать ровно одну дату из предложенных. При успехе выставляет `status=accepted`, заполняет `selected_date` и создаёт записи занятости (`busyday`) для обоих пользователей.",
    params(
        ("id" = Uuid, Path, description = "Invitation id")
    ),
    request_body = AcceptInvitationRequest,
    responses(
        (status = 200, description = "Invitation accepted", body = InvitationResponse),
        (status = 400, description = "selected_date invalid"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invitation not found"),
        (status = 409, description = "Invitation already resolved or day is busy")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn accept_invitation(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<Json<InvitationResponse>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let selected_date = parse_one_date(&payload.selected_date)?;
    let transaction = db.begin().await.map_err(internal_error)?;

    let invitation = Invitation::find_by_id(id)
        .filter(InvitationColumn::ToUserId.eq(me))
        .filter(InvitationColumn::Status.eq(InvitationStatus::Pending))
        .one(&transaction)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "invitation not found".to_string()))?;

    let has_selected_date = InvitationDate::find()
        .filter(InvitationDateColumn::InvitationId.eq(id))
        .filter(InvitationDateColumn::Date.eq(selected_date))
        .one(&transaction)
        .await
        .map_err(internal_error)?
        .is_some();

    if !has_selected_date {
        return Err((
            StatusCode::BAD_REQUEST,
            "selected_date is not proposed".to_string(),
        ));
    }

    ensure_day_is_free(&transaction, invitation.from_user_id, selected_date).await?;
    ensure_day_is_free(&transaction, invitation.to_user_id, selected_date).await?;

    let from_user_id = invitation.from_user_id;
    let to_user_id = invitation.to_user_id;
    let mut active = invitation.into_active_model();

    active.status = Set(InvitationStatus::Accepted);
    active.selected_date = Set(Some(selected_date));
    active
        .update(&transaction)
        .await
        .map_err(internal_error)?;

    BusydayActiveModel {
        user_id: Set(from_user_id),
        date: Set(selected_date),
        event_id: Set(None),
        ..Default::default()
    }
    .insert(&transaction)
    .await
    .map_err(map_db_constraint_error)?;

    BusydayActiveModel {
        user_id: Set(to_user_id),
        date: Set(selected_date),
        event_id: Set(None),
        ..Default::default()
    }
    .insert(&transaction)
    .await
    .map_err(map_db_constraint_error)?;

    transaction
        .commit()
        .await
        .map_err(internal_error)?;

    Ok(Json(load_invitation_response(&db, id).await?))
}

#[utoipa::path(
    post,
    path = "/invitations/{id}/decline",
    summary = "Отклонить приглашение",
    description = "Отклоняет приглашение в статусе `pending`. Действие доступно только получателю. Возвращает обновлённое приглашение со статусом `declined`.",
    params(
        ("id" = Uuid, Path, description = "Invitation id")
    ),
    responses(
        (status = 200, description = "Invitation declined", body = InvitationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invitation not found"),
        (status = 409, description = "Invitation already resolved")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn decline_invitation(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvitationResponse>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;

    let invitation = Invitation::find_by_id(id)
        .filter(InvitationColumn::ToUserId.eq(me))
        .filter(InvitationColumn::Status.eq(InvitationStatus::Pending))
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "invitation not found".to_string()))?;

    let mut active = invitation.into_active_model();
    active.status = Set(InvitationStatus::Declined);
    active
        .update(&db)
        .await
        .map_err(internal_error)?;

    Ok(Json(load_invitation_response(&db, id).await?))
}

#[utoipa::path(
    post,
    path = "/invitations/{id}/cancel",
    summary = "Отменить приглашение",
    description = "Отменяет (удаляет) приглашение в статусе `pending`. Действие доступно только отправителю. При успехе возвращает `204 No Content`.",
    params(
        ("id" = Uuid, Path, description = "Invitation id")
    ),
    responses(
        (status = 204, description = "Invitation cancelled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invitation not found"),
        (status = 409, description = "Invitation already resolved")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn cancel_invitation(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;

    let invitation = Invitation::find_by_id(id)
        .filter(InvitationColumn::FromUserId.eq(me))
        .filter(InvitationColumn::Status.eq(InvitationStatus::Pending))
        .one(&db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "invitation not found".to_string()))?;

    invitation
        .into_active_model()
        .delete(&db)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// TODO: прочекать отсюда
#[utoipa::path(
    get,
    path = "/users/{user_id}/calendar",
    summary = "Календарь пользователя",
    description = "Возвращает данные календаря за диапазон `from..to`: занятые дни (`busy_days`), даты по приглашениям в статусе `pending` (`pending_invites`) и прошедшие занятые дни (`past_events`). Доступ: сам пользователь или его принятый друг.",
    params(
        ("user_id" = Uuid, Path, description = "User id"),
        CalendarQuery
    ),
    responses(
        (status = 200, description = "Calendar data", body = CalendarResponse),
        (status = 400, description = "Invalid date range"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Calendar"
)]
pub async fn get_user_calendar(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    if me != user_id && !are_users_accepted_friends(&db, me, user_id).await? {
        return Err((StatusCode::FORBIDDEN, "access denied".to_string()));
    }

    let (from, to) = parse_date_range(&query.from, &query.to)?;

    let busy_rows = Busyday::find()
        .filter(BusydayColumn::UserId.eq(user_id))
        .filter(BusydayColumn::Date.gte(from))
        .filter(BusydayColumn::Date.lte(to))
        .order_by_asc(BusydayColumn::Date)
        .all(&db)
        .await
        .map_err(internal_error)?;

    let busy_days: Vec<BusydayResponse> = busy_rows
        .iter()
        .map(|row| BusydayResponse {
            id: row.id,
            user_id: row.user_id,
            date: row.date.to_string(),
            event_id: row.event_id,
        })
        .collect();

    let today = Utc::now().date_naive();
    let past_events = busy_rows
        .iter()
        .filter(|row| row.date < today)
        .map(|row| row.date.to_string())
        .collect::<Vec<_>>();

    let pending_invitations = Invitation::find()
        .filter(InvitationColumn::Status.eq(InvitationStatus::Pending))
        .filter(
            Condition::any()
                .add(InvitationColumn::FromUserId.eq(user_id))
                .add(InvitationColumn::ToUserId.eq(user_id)),
        )
        .all(&db)
        .await
        .map_err(internal_error)?;

    let mut direction_by_invitation = HashMap::new();
    let invitation_ids: Vec<Uuid> = pending_invitations
        .into_iter()
        .map(|invitation| {
            let direction = if invitation.to_user_id == user_id {
                "incoming"
            } else {
                "outgoing"
            };
            direction_by_invitation.insert(invitation.id, direction.to_string());
            invitation.id
        })
        .collect();

    let mut pending_invites = Vec::new();
    if !invitation_ids.is_empty() {
        let pending_dates = InvitationDate::find()
            .filter(InvitationDateColumn::InvitationId.is_in(invitation_ids))
            .filter(InvitationDateColumn::Date.gte(from))
            .filter(InvitationDateColumn::Date.lte(to))
            .all(&db)
            .await
            .map_err(internal_error)?;

        pending_invites = pending_dates
            .into_iter()
            .filter_map(|row| {
                direction_by_invitation
                    .get(&row.invitation_id)
                    .map(|direction| PendingInviteResponse {
                        invitation_id: row.invitation_id,
                        date: row.date.to_string(),
                        direction: direction.clone(),
                    })
            })
            .collect();
    }

    Ok(Json(CalendarResponse {
        from: from.to_string(),
        to: to.to_string(),
        busy_days,
        pending_invites,
        past_events,
    }))
}

#[utoipa::path(
    get,
    path = "/users/me/busydays",
    summary = "Мои занятые дни",
    description = "Возвращает занятые дни текущего пользователя за диапазон дат `from..to`. Используется для отрисовки личной занятости в календаре.",
    params(CalendarQuery),
    responses(
        (status = 200, description = "Current user busy days", body = [BusydayResponse]),
        (status = 400, description = "Invalid date range"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Calendar"
)]
pub async fn get_my_busydays(
    auth: AuthUser,
    State(db): State<DatabaseConnection>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<Vec<BusydayResponse>>, (StatusCode, String)> {
    let me = parse_auth_user_id(auth)?;
    let (from, to) = parse_date_range(&query.from, &query.to)?;

    let rows = Busyday::find()
        .filter(BusydayColumn::UserId.eq(me))
        .filter(BusydayColumn::Date.gte(from))
        .filter(BusydayColumn::Date.lte(to))
        .order_by_asc(BusydayColumn::Date)
        .all(&db)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        rows.into_iter()
            .map(|row| BusydayResponse {
                id: row.id,
                user_id: row.user_id,
                date: row.date.to_string(),
                event_id: row.event_id,
            })
            .collect(),
    ))
}

async fn load_invitation_response(
    db: &DatabaseConnection,
    invitation_id: Uuid,
) -> Result<InvitationResponse, (StatusCode, String)> {
    let invitation = Invitation::find_by_id(invitation_id)
        .one(db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "invitation not found".to_string()))?;

    let dates = InvitationDate::find()
        .filter(InvitationDateColumn::InvitationId.eq(invitation_id))
        .order_by_asc(InvitationDateColumn::Date)
        .all(db)
        .await
        .map_err(internal_error)?;

    let status = match invitation.status {
        InvitationStatus::Pending => "pending",
        InvitationStatus::Accepted => "accepted",
        InvitationStatus::Declined => "declined",
    };

    Ok(InvitationResponse {
        id: invitation.id,
        from_user_id: invitation.from_user_id,
        to_user_id: invitation.to_user_id,
        status: status.to_string(),
        selected_date: invitation.selected_date.map(|d| d.to_string()),
        created_at: invitation.created_at.to_rfc3339(),
        dates: dates
            .into_iter()
            .map(|d| InvitationDateResponse {
                id: d.id,
                invitation_id: d.invitation_id,
                date: d.date.to_string(),
            })
            .collect(),
    })
}

async fn ensure_day_is_free(
    tx: &DatabaseTransaction,
    user_id: Uuid,
    date: NaiveDate,
) -> Result<(), (StatusCode, String)> {
    let exists = Busyday::find()
        .filter(BusydayColumn::UserId.eq(user_id))
        .filter(BusydayColumn::Date.eq(date))
        .one(tx)
        .await
        .map_err(internal_error)?
        .is_some();
    if exists {
        return Err((
            StatusCode::CONFLICT,
            "selected day is already busy".to_string(),
        ));
    }
    Ok(())
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

fn parse_date_range(from: &str, to: &str) -> Result<(NaiveDate, NaiveDate), (StatusCode, String)> {
    let from_date = parse_one_date(from)?;
    let to_date = parse_one_date(to)?;
    if from_date > to_date {
        return Err((
            StatusCode::BAD_REQUEST,
            "`from` must be <= `to`".to_string(),
        ));
    }
    Ok((from_date, to_date))
}

fn parse_and_validate_dates(dates: &[String]) -> Result<Vec<NaiveDate>, (StatusCode, String)> {
    if dates.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "dates cannot be empty".to_string()));
    }

    let mut parsed = Vec::with_capacity(dates.len());
    let mut uniq = HashSet::new();
    let today = Utc::now().date_naive();
    for raw in dates {
        let day = parse_one_date(raw)?;
        if day < today {
            return Err((
                StatusCode::BAD_REQUEST,
                "past dates are not allowed".to_string(),
            ));
        }
        if !uniq.insert(day) {
            return Err((StatusCode::BAD_REQUEST, "dates must be unique".to_string()));
        }
        parsed.push(day);
    }
    Ok(parsed)
}

fn parse_one_date(value: &str) -> Result<NaiveDate, (StatusCode, String)> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid date format `{value}`, expected YYYY-MM-DD"),
        )
    })
}

fn validate_create_invitation_payload(
    me: Uuid,
    payload: &CreateInvitationRequest,
) -> Result<(), (StatusCode, String)> {
    if payload.to_user_id == me {
        return Err((
            StatusCode::BAD_REQUEST,
            "cannot invite yourself".to_string(),
        ));
    }
    if payload.dates.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "dates cannot be empty".to_string()));
    }
    Ok(())
}

fn parse_auth_user_id(auth: AuthUser) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(&auth.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))
}

fn map_db_constraint_error(err: sea_orm::DbErr) -> (StatusCode, String) {
    let message = err.to_string();
    if message.contains("unique")
        || message.contains("duplicate key")
        || message.contains("idx_busydays_user_date_unique")
        || message.contains("idx_invitation_dates_invitation_date_unique")
    {
        return (StatusCode::CONFLICT, message);
    }
    internal_error(err)
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
