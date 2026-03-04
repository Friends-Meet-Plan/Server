use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateWishPlaceBody {
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub link: Option<String>,
}
