use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct PendingInviteResponse {
    pub invitation_id: Uuid,
    pub date: String,
    pub direction: String,
}
