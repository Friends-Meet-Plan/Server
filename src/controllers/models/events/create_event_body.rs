use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateEventBody {
    pub date: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub participant_ids: Vec<String>,
    pub wish_place_id: Option<Uuid>,
}