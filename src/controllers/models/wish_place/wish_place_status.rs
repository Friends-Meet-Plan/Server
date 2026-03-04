use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WishPlaceStatusDto {
    Active,
    Visited,
    Archived,
}
