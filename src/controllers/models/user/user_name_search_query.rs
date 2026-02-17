use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct UserNameSearchQuery {
    pub username: Option<String>,
}
