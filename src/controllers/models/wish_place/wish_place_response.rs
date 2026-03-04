use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct WishPlaceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub link: Option<String>,
    pub status: String,
    pub visited_event_id: Option<Uuid>,
    pub created_at: String,
}
