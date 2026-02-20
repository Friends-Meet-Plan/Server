use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateInvitationRequest {
    #[schema(example = "6a34a410-57c6-4f88-946f-7f1373799da2")]
    pub to_user_id: Uuid,
    pub dates: Vec<String>,
}
