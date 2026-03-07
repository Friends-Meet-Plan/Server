use std::fmt;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "event_status")]
pub enum EventStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "confirmed")]
    Confirmed,
    #[sea_orm(string_value = "canceled")]
    Canceled,
    #[sea_orm(string_value = "completed")]
    Completed,
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EventStatus::Pending => "pending",
            EventStatus::Confirmed => "confirmed",
            EventStatus::Canceled => "canceled",
            EventStatus::Completed => "completed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub creator_id: Uuid,
    pub date: Date,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub status: EventStatus,
    pub wish_place_id: Option<Uuid>,
    pub memory_image_base64: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
