use uuid::Uuid;
use utoipa::ToSchema;

#[derive(serde::Serialize, ToSchema)]
pub struct ParticipantResponse {
    pub user_id: Uuid,
    pub status: String,
}
