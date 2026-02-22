use serde::Serialize;
use utoipa::ToSchema;

use crate::controllers::models::BusydayResponse;

#[derive(Serialize, ToSchema)]
pub struct CalendarResponse {
    pub from: String,
    pub to: String,
    pub busy_days: Vec<BusydayResponse>,
    pub past_events: Vec<String>,
}
