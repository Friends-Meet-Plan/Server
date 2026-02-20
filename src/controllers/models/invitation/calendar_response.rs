use serde::Serialize;
use utoipa::ToSchema;

use crate::controllers::models::{BusydayResponse, PendingInviteResponse};

#[derive(Serialize, ToSchema)]
pub struct CalendarResponse {
    pub from: String,
    pub to: String,
    pub busy_days: Vec<BusydayResponse>,
    pub pending_invites: Vec<PendingInviteResponse>,
    pub past_events: Vec<String>,
}
