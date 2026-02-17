use serde::Serialize;
use utoipa::ToSchema;

use crate::controllers::models::user_response::UserResponse;

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}
