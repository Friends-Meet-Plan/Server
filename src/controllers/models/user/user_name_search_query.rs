use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserNameSearchQuery {
    pub username: Option<String>,
}
