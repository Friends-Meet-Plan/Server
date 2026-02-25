use crate::migration::uuid_pk;
use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'wish_place_status') THEN\n        CREATE TYPE wish_place_status AS ENUM ('active', 'visited', 'archived');\n    END IF;\nEND$$;"
                    .to_string(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WishPlaces::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(WishPlaces::UserId).uuid().not_null())
                    .col(ColumnDef::new(WishPlaces::Title).string().not_null())
                    .col(ColumnDef::new(WishPlaces::Description).string().null())
                    .col(ColumnDef::new(WishPlaces::Location).string().null())
                    .col(ColumnDef::new(WishPlaces::Link).string().null())
                    .col(
                        ColumnDef::new(WishPlaces::Status)
                            .enumeration(
                                WishPlaceStatus::Table,
                                [
                                    WishPlaceStatus::Active,
                                    WishPlaceStatus::Visited,
                                    WishPlaceStatus::Archived,
                                ],
                            )
                            .not_null()
                            .default(WishPlaceStatus::Active.to_string()),
                    )
                    .col(ColumnDef::new(WishPlaces::VisitedEventId).uuid().null())
                    .col(
                        ColumnDef::new(WishPlaces::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(WishPlaces::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wish_places_user_id")
                            .from(WishPlaces::Table, WishPlaces::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wish_places_visited_event_id")
                            .from(WishPlaces::Table, WishPlaces::VisitedEventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wish_places_user_id")
                    .table(WishPlaces::Table)
                    .col(WishPlaces::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_wish_places_status")
                    .table(WishPlaces::Table)
                    .col(WishPlaces::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_wish_places_status")
                    .table(WishPlaces::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_wish_places_user_id")
                    .table(WishPlaces::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(WishPlaces::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE IF EXISTS wish_place_status".to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum WishPlaces {
    Table,
    Id,
    UserId,
    Title,
    Description,
    Location,
    Link,
    Status,
    VisitedEventId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Events {
    Table,
    Id,
}

#[derive(Iden)]
enum WishPlaceStatus {
    #[iden = "wish_place_status"]
    Table,
    #[iden = "active"]
    Active,
    #[iden = "visited"]
    Visited,
    #[iden = "archived"]
    Archived,
}
