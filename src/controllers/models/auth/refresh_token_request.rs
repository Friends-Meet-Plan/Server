use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
