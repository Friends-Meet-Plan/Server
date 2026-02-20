use crate::migration::uuid_pk;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InvitationDates::Table)
                    .if_not_exists()
                    .col(uuid_pk())
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_invitation_dates_invitation_date_unique")
                    .table(InvitationDates::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(InvitationDates::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum InvitationDates {
    Table,
    Id,
    InvitationId,
    Date,
}

#[derive(Iden)]
enum Invitations {
    Table,
    Id,
}
