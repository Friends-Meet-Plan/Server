use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AuthRequestBody {
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub password: String,
}
