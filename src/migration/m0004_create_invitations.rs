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
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'invitation_status') THEN\n        CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'declined');\n    END IF;\nEND$$;"
                    .to_string(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Invitations::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(Invitations::FromUserId).uuid().not_null())
                    .col(ColumnDef::new(Invitations::ToUserId).uuid().not_null())
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
                            .not_null()
                            .default(InvitationStatus::Pending.to_string()),
                    )
                    .col(ColumnDef::new(Invitations::SelectedDate).date().null())
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
                    .check(
                        Expr::cust(
                            "\"from_user_id\" <> \"to_user_id\" AND (\"selected_date\" IS NULL OR \"status\" = 'accepted'::invitation_status)",
                        ),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_from_user_id")
                    .table(Invitations::Table)
                    .col(Invitations::FromUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_to_user_id")
                    .table(Invitations::Table)
                    .col(Invitations::ToUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_status")
                    .table(Invitations::Table)
                    .col(Invitations::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE OR REPLACE FUNCTION ensure_invitations_are_friends()\nRETURNS TRIGGER AS $$\nBEGIN\n    IF NOT EXISTS (\n        SELECT 1\n        FROM friendships f\n        WHERE (\n            (f.user_id = NEW.from_user_id AND f.friend_id = NEW.to_user_id)\n            OR\n            (f.user_id = NEW.to_user_id AND f.friend_id = NEW.from_user_id)\n        )\n        AND f.status = 'accepted'::friendship_status\n    ) THEN\n        RAISE EXCEPTION 'Invitation requires accepted friendship between users';\n    END IF;\n\n    RETURN NEW;\nEND;\n$$ LANGUAGE plpgsql;"
                    .to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TRIGGER IF EXISTS trg_invitations_ensure_friends ON invitations"
                    .to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE TRIGGER trg_invitations_ensure_friends\nBEFORE INSERT OR UPDATE OF from_user_id, to_user_id\nON invitations\nFOR EACH ROW\nEXECUTE FUNCTION ensure_invitations_are_friends();"
                    .to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TRIGGER IF EXISTS trg_invitations_ensure_friends ON invitations".to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP FUNCTION IF EXISTS ensure_invitations_are_friends()".to_string(),
            ))
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_invitations_status")
                    .table(Invitations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_invitations_to_user_id")
                    .table(Invitations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_invitations_from_user_id")
                    .table(Invitations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Invitations::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE IF EXISTS invitation_status".to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Invitations {
    Table,
    Id,
    FromUserId,
    ToUserId,
    Status,
    SelectedDate,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum InvitationStatus {
    #[iden = "invitation_status"]
    Table,
    #[iden = "pending"]
    Pending,
    #[iden = "accepted"]
    Accepted,
    #[iden = "declined"]
    Declined,
}
