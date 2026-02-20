use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct CalendarQuery {
    pub from: String,
    pub to: String,
}
