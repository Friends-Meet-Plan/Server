use serde::Deserialize;
use utoipa::ToSchema;

use crate::controllers::models::wish_place::WishPlaceStatusDto;

#[derive(Deserialize, ToSchema)]
pub struct UpdateWishPlaceBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub link: Option<String>,
    pub status: Option<WishPlaceStatusDto>,
}
