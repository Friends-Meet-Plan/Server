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
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'event_status') THEN\n        CREATE TYPE event_status AS ENUM ('pending', 'confirmed', 'canceled', 'completed');\n    END IF;\nEND$$;"
                    .to_string(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Events::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(Events::CreatorId).uuid().not_null())
                    .col(ColumnDef::new(Events::Date).date().not_null())
                    .col(ColumnDef::new(Events::Title).string().not_null())
                    .col(ColumnDef::new(Events::Description).string().null())
                    .col(ColumnDef::new(Events::Location).string().null())
                    .col(
                        ColumnDef::new(Events::Status)
                            .enumeration(
                                EventStatus::Table,
                                [
                                    EventStatus::Pending,
                                    EventStatus::Confirmed,
                                    EventStatus::Canceled,
                                    EventStatus::Completed,
                                ],
                            )
                            .not_null()
                            .default(EventStatus::Pending.to_string()),
                    )
                    .col(ColumnDef::new(Events::WishPlaceId).uuid().null())
                    .col(
                        ColumnDef::new(Events::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_events_creator_id")
                            .from(Events::Table, Events::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_events_creator_id")
                    .table(Events::Table)
                    .col(Events::CreatorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_events_date")
                    .table(Events::Table)
                    .col(Events::Date)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_events_date")
                    .table(Events::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_events_creator_id")
                    .table(Events::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE IF EXISTS event_status".to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Events {
    Table,
    Id,
    CreatorId,
    Date,
    Title,
    Description,
    Location,
    Status,
    WishPlaceId,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum EventStatus {
    #[iden = "event_status"]
    Table,
    #[iden = "pending"]
    Pending,
    #[iden = "confirmed"]
    Confirmed,
    #[iden = "canceled"]
    Canceled,
    #[iden = "completed"]
    Completed,
}
