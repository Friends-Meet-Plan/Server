use std::fmt;
use std::fmt::Display;
use sea_orm::entity::prelude::*;
use crate::entities::EventParticipant;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "event_participant_status")]
pub enum EventParticipantStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "declined")]
    Declined,
}

impl fmt::Display for EventParticipantStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self { 
            EventParticipantStatus::Pending => "pending",
            EventParticipantStatus::Accepted => "accepted",
            EventParticipantStatus::Declined => "declined",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_participants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: EventParticipantStatus,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}