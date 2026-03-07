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
                "DROP TABLE IF EXISTS invitation_dates".to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "ALTER TABLE invitations DROP COLUMN IF EXISTS selected_date".to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DELETE FROM invitations".to_string(),
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitations::Table)
                    .add_column(ColumnDef::new(Invitations::EventId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_event_id")
                    .table(Invitations::Table)
                    .col(Invitations::EventId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitations::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_invitations_event_id")
                            .from_tbl(Invitations::Table)
                            .from_col(Invitations::EventId)
                            .to_tbl(Events::Table)
                            .to_col(Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "ALTER TABLE invitations DROP CONSTRAINT IF EXISTS invitations_check".to_string(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "ALTER TABLE invitations ADD CONSTRAINT invitations_no_self_check CHECK (from_user_id <> to_user_id)"
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
                "ALTER TABLE invitations DROP CONSTRAINT IF EXISTS invitations_no_self_check"
                    .to_string(),
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitations::Table)
                    .drop_foreign_key(Alias::new("fk_invitations_event_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_invitations_event_id")
                    .table(Invitations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitations::Table)
                    .drop_column(Invitations::EventId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitations::Table)
                    .add_column(ColumnDef::new(Invitations::SelectedDate).date().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(InvitationDates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InvitationDates::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(InvitationDates::InvitationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InvitationDates::Date).date().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invitation_dates_invitation_id")
                            .from(InvitationDates::Table, InvitationDates::InvitationId)
                            .to(Invitations::Table, Invitations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitation_dates_invitation_date_unique")
                    .table(InvitationDates::Table)
                    .col(InvitationDates::InvitationId)
                    .col(InvitationDates::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "ALTER TABLE invitations ADD CONSTRAINT invitations_check CHECK (from_user_id <> to_user_id AND (selected_date IS NULL OR status = 'accepted'::invitation_status))"
                    .to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Invitations {
    Table,
    Id,
    EventId,
    SelectedDate,
}

#[derive(Iden)]
enum InvitationDates {
    Table,
    Id,
    InvitationId,
    Date,
}

#[derive(Iden)]
enum Events {
    Table,
    Id,
}
