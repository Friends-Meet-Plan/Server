use sea_orm::entity::prelude::*;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "wish_place_status")]
pub enum WishPlaceStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "visited")]
    Visited,
    #[sea_orm(string_value = "archived")]
    Archived,
}

impl fmt::Display for WishPlaceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WishPlaceStatus::Active => "active",
            WishPlaceStatus::Visited => "visited",
            WishPlaceStatus::Archived => "archived",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wish_places")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub link: Option<String>,
    pub status: WishPlaceStatus,
    pub visited_event_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
