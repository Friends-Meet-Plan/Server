use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthRequestBody {
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub password: String,
}
