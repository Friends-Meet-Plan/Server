use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct VisitWishPlaceBody {
    pub event_id: Uuid,
}
