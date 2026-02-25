use uuid::Uuid;
use crate::controllers::models::events::ParticipantResponse;

#[derive(serde::Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub date: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub status: String,
    pub wish_place_id: Option<Uuid>,
    pub created_at: String,
    pub participants: Vec<ParticipantResponse>,
}