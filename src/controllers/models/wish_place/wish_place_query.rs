use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Deserialize, IntoParams)]
pub struct WishPlaceQuery {
    pub user_id: Uuid,
}
