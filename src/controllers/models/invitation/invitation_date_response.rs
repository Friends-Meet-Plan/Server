use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct InvitationDateResponse {
    pub id: Uuid,
    pub invitation_id: Uuid,
    pub date: String,
}
