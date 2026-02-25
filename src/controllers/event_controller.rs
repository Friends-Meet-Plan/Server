use sea_orm::{ColumnTrait, Condition, QueryOrder};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;
use crate::auth::middleware::AuthUser;
use crate::controllers::models::events::{CreateEventBody, EventResponse, ParticipantResponse};
use crate::entities::event::EventStatus;
use crate::entities::{event, EventActiveModel, EventParticipant, EventParticipantActiveModel, EventParticipantColumn};
use crate::entities::event_participant::EventParticipantStatus;
use crate::entities::friendship::{self, FriendshipStatus};
use crate::entities::{Friendship, FriendshipColumn};

pub fn router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/events", axum::routing::post(create_event))
}

#[utoipa::path(
    post,
    path = "/events",
    summary = "Создать событие",
    description = "Создаёт событие, автоматически добавляет creator как accepted участника и добавляет остальных участников из `participant_ids` со статусом `pending`.",
    request_body = CreateEventBody,
    responses(
        (status = 201, description = "Event created", body = EventResponse),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Events"
)]
pub async fn create_event(
    auth: AuthUser,
    State(db_connection): State<DatabaseConnection>,
    Json(body): Json<CreateEventBody>,
) -> Result<(StatusCode, Json<EventResponse>), (StatusCode, String)> {
    let me_id = parse_auth(auth)?;
    let date = parse_date(&body.date)?;
    let mut participant_ids = body.participant_ids.clone();

    participant_ids.retain(|id| id != &me_id.to_string());
    participant_ids.sort();
    participant_ids.dedup();

    let mut parsed_participant_ids = Vec::with_capacity(participant_ids.len());
    for participant_id in participant_ids {
        let p_id = Uuid::parse_str(&participant_id)
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid participant id".to_string()))?;

        let is_friend = Friendship::find()
            .filter(FriendshipColumn::Status.eq(FriendshipStatus::Accepted))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(friendship::Column::UserId.eq(me_id))
                            .add(friendship::Column::FriendId.eq(p_id)),
                    )
                    .add(
                        Condition::all()
                            .add(friendship::Column::UserId.eq(p_id))
                            .add(friendship::Column::FriendId.eq(me_id)),
                    ),
            )
            .one(&db_connection)
            .await
            .map_err(internal_error)?
            .is_some();

        if !is_friend {
            return Err((
                StatusCode::FORBIDDEN,
                "can invite only accepted friends".to_string(),
            ));
        }

        parsed_participant_ids.push(p_id);
    }

    let transaction = db_connection
        .begin()
        .await
        .map_err(|e| internal_error(format!("DB connection error: {}", e)))?;

    let event = EventActiveModel {
        creator_id: Set(me_id),
        date: Set(date),
        title: Set(body.title),
        description: Set(body.description),
        location: Set(body.location),
        status: Set(EventStatus::Pending),
        wish_place_id: Set(body.wish_place_id),
        ..Default::default()
    }
    .insert(&transaction)
    .await
    .map_err(|e| internal_error(format!("DB connection error: {}", e)))?;

    EventParticipantActiveModel {
        event_id: Set(event.id),
        user_id: Set(me_id),
        status: Set(EventParticipantStatus::Accepted),
        ..Default::default()
    }
    .insert(&transaction)
    .await
    .map_err(|e| internal_error(format!("DB connection error: {}", e)))?;

    let mut models = Vec::with_capacity(parsed_participant_ids.len());

    for p_id in parsed_participant_ids {
        models.push(
            EventParticipantActiveModel {
                event_id: Set(event.id),
                user_id: Set(p_id),
                status: Set(EventParticipantStatus::Pending),
                ..Default::default()
            }
        );
    }

    EventParticipant::insert_many(models)
        .exec(&transaction)
        .await
        .map_err(internal_error)?;

    transaction
        .commit()
        .await
        .map_err(|e| internal_error(e.to_string()))?;

    let response = load_event_response(&db_connection, event.id).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// MARK: Helper
fn parse_auth(auth: AuthUser) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(&auth.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user id".to_string()))
}

fn parse_date(value: &str) -> Result<NaiveDate, (StatusCode, String)> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid date".to_string()))
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

async fn load_event_response(
    db: &DatabaseConnection,
    event_id: Uuid,
) -> Result<EventResponse, (StatusCode, String)> {
    let event = event::Entity::find_by_id(event_id)
        .one(db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "event not found".to_string()))?;

    let participants = EventParticipant::find()
        .filter(EventParticipantColumn::EventId.eq(event_id))
        .order_by_asc(EventParticipantColumn::UserId)
        .all(db)
        .await
        .map_err(internal_error)?;

    let participants = participants
        .into_iter()
        .map(|p| ParticipantResponse {
            user_id: p.user_id,
            status: p.status.to_string(),
        })
        .collect();

    Ok(EventResponse {
        id: event.id,
        creator_id: event.creator_id,
        date: event.date.to_string(),
        title: event.title,
        description: event.description,
        location: event.location,
        status: event.status.to_string(),
        wish_place_id: event.wish_place_id,
        created_at: event.created_at.to_rfc3339(),
        participants,
    })
}
