use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct IsBusyRequest {
    pub id: Uuid,
    #[schema(example = "2026-03-04")]
    pub date: String,
}
