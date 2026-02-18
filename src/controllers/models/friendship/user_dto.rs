use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserDTO {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}