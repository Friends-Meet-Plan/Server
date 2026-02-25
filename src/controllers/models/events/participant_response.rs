use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct ParticipantResponse {
    pub user_id: Uuid,
    pub status: String,
}