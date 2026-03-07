use std::fmt;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_event_role")]
pub enum UserEventRole {
    #[sea_orm(string_value = "owner")]
    Owner,
    #[sea_orm(string_value = "participant")]
    Participant,
}

impl fmt::Display for UserEventRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            UserEventRole::Owner => "owner",
            UserEventRole::Participant => "participant",
        };
        write!(f, "{}", value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "user_event_response"
)]
pub enum UserEventResponse {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "declined")]
    Declined,
}

impl fmt::Display for UserEventResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            UserEventResponse::Pending => "pending",
            UserEventResponse::Accepted => "accepted",
            UserEventResponse::Declined => "declined",
        };
        write!(f, "{}", value)
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub role: UserEventRole,
    pub response_status: UserEventResponse,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
