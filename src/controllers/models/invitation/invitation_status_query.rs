use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatusFilter {
    Pending,
    Accepted,
    Declined,
}

#[derive(Deserialize, IntoParams)]
pub struct InvitationStatusQuery {
    #[param(example = "pending")]
    pub status: Option<InvitationStatusFilter>,
}
