use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateUserRequestBody {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}
