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
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'event_participant_status') THEN\n        CREATE TYPE event_participant_status AS ENUM ('pending', 'accepted', 'declined');\n    END IF;\nEND$$;"
                    .to_string(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EventParticipants::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(EventParticipants::EventId).uuid().not_null())
                    .col(ColumnDef::new(EventParticipants::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(EventParticipants::Status)
                            .enumeration(
                                EventParticipantStatus::Table,
                                [
                                    EventParticipantStatus::Pending,
                                    EventParticipantStatus::Accepted,
                                    EventParticipantStatus::Declined,
                                ],
                            )
                            .not_null()
                            .default(EventParticipantStatus::Pending.to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_participants_event_id")
                            .from(EventParticipants::Table, EventParticipants::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_participants_user_id")
                            .from(EventParticipants::Table, EventParticipants::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_participants_event_user_unique")
                    .table(EventParticipants::Table)
                    .col(EventParticipants::EventId)
                    .col(EventParticipants::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_participants_event_id")
                    .table(EventParticipants::Table)
                    .col(EventParticipants::EventId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_participants_user_id")
                    .table(EventParticipants::Table)
                    .col(EventParticipants::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_event_participants_user_id")
                    .table(EventParticipants::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_event_participants_event_id")
                    .table(EventParticipants::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_event_participants_event_user_unique")
                    .table(EventParticipants::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(EventParticipants::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE IF EXISTS event_participant_status".to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum EventParticipants {
    Table,
    Id,
    EventId,
    UserId,
    Status,
}

#[derive(Iden)]
enum Events {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum EventParticipantStatus {
    #[iden = "event_participant_status"]
    Table,
    #[iden = "pending"]
    Pending,
    #[iden = "accepted"]
    Accepted,
    #[iden = "declined"]
    Declined,
}
