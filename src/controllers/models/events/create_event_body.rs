use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateEventBody {
    #[schema(example = "2026-03-03")]
    pub date: String,
    #[schema(example = "Coffee meetup")]
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub participant_ids: Vec<Uuid>,
    pub wish_place_id: Option<Uuid>,
}
