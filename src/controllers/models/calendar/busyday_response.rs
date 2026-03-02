use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct BusydayResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub date: String,
    pub event_id: Option<Uuid>,
}
