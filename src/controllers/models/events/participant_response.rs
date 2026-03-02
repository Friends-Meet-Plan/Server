use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Serialize, ToSchema)]
pub struct ParticipantResponse {
    pub user_id: Uuid,
    pub role: String,
    pub response_status: String,
}
