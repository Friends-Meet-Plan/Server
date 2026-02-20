use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct CreatedInvitationResponse {
    pub id: Uuid,
}
