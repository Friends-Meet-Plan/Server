use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequestBody {
    pub username: String,
    pub password: String,
}
