use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::controllers::models::InvitationDateResponse;

#[derive(Serialize, ToSchema)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub status: String,
    pub selected_date: Option<String>,
    pub created_at: String,
    pub dates: Vec<InvitationDateResponse>,
}
