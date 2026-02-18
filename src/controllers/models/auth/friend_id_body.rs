use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FriendIdBody {
    pub id: Uuid,
}
