use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UpdateEventBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}
