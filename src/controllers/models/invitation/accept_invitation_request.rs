use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    #[schema(example = "2026-02-26")]
    pub selected_date: String,
}
