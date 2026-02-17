use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequestBody {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}
