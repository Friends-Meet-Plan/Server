use crate::migration::uuid_pk;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_event_role') THEN\n        CREATE TYPE user_event_role AS ENUM ('owner', 'participant');\n    END IF;\nEND$$;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_event_response') THEN\n        CREATE TYPE user_event_response AS ENUM ('pending', 'accepted', 'declined');\n    END IF;\nEND$$;",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserEvents::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(UserEvents::EventId).uuid().not_null())
                    .col(ColumnDef::new(UserEvents::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserEvents::Role)
                            .enumeration(
                                UserEventRole::Table,
                                [UserEventRole::Owner, UserEventRole::Participant],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserEvents::ResponseStatus)
                            .enumeration(
                                UserEventResponse::Table,
                                [
                                    UserEventResponse::Pending,
                                    UserEventResponse::Accepted,
                                    UserEventResponse::Declined,
                                ],
                            )
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_events_event_id")
                            .from(UserEvents::Table, UserEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_events_user_id")
                            .from(UserEvents::Table, UserEvents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_events_event_user_unique")
                    .table(UserEvents::Table)
                    .col(UserEvents::EventId)
                    .col(UserEvents::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_events_user_status")
                    .table(UserEvents::Table)
                    .col(UserEvents::UserId)
                    .col(UserEvents::ResponseStatus)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_events_event_id")
                    .table(UserEvents::Table)
                    .col(UserEvents::EventId)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS invitation_dates")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS invitations")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS invitation_status")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'invitation_status') THEN\n        CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'declined');\n    END IF;\nEND$$;",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Invitations::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(Invitations::FromUserId).uuid().not_null())
                    .col(ColumnDef::new(Invitations::ToUserId).uuid().not_null())
                    .col(ColumnDef::new(Invitations::EventId).uuid().not_null())
                    .col(
                        ColumnDef::new(Invitations::Status)
                            .enumeration(
                                InvitationStatus::Table,
                                [
                                    InvitationStatus::Pending,
                                    InvitationStatus::Accepted,
                                    InvitationStatus::Declined,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Invitations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invitations_from_user_id")
                            .from(Invitations::Table, Invitations::FromUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invitations_to_user_id")
                            .from(Invitations::Table, Invitations::ToUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invitations_event_id")
                            .from(Invitations::Table, Invitations::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .check(Expr::cust("from_user_id <> to_user_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_events_event_id")
                    .table(UserEvents::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_events_user_status")
                    .table(UserEvents::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_events_event_user_unique")
                    .table(UserEvents::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserEvents::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS user_event_response")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS user_event_role")
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum UserEvents {
    Table,
    Id,
    EventId,
    UserId,
    Role,
    ResponseStatus,
}

#[derive(Iden)]
enum UserEventRole {
    #[iden = "user_event_role"]
    Table,
    Owner,
    Participant,
}

#[derive(Iden)]
enum UserEventResponse {
    #[iden = "user_event_response"]
    Table,
    Pending,
    Accepted,
    Declined,
}

#[derive(Iden)]
enum Invitations {
    Table,
    Id,
    FromUserId,
    ToUserId,
    EventId,
    Status,
    CreatedAt,
}

#[derive(Iden)]
enum InvitationStatus {
    #[iden = "invitation_status"]
    Table,
    Pending,
    Accepted,
    Declined,
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
