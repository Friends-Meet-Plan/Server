use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EventScope {
    Created,
    Invited,
    Upcoming,
    Past,
}

#[derive(Deserialize, IntoParams)]
pub struct EventScopeQuery {
    pub scope: Option<EventScope>,
}
