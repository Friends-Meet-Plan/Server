use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct PendingInviteResponse {
    pub event_id: Uuid,
    pub date: String,
}
