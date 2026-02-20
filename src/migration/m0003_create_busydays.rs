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
                    .table(Busydays::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(Busydays::UserId).uuid().not_null())
                    .col(ColumnDef::new(Busydays::Date).date().not_null())
                    .col(ColumnDef::new(Busydays::EventId).uuid().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_busydays_user_id")
                            .from(Busydays::Table, Busydays::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_busydays_user_date_unique")
                    .table(Busydays::Table)
                    .col(Busydays::UserId)
                    .col(Busydays::Date)
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
                    .name("idx_busydays_user_date_unique")
                    .table(Busydays::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Busydays::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Busydays {
    Table,
    Id,
    UserId,
    Date,
    EventId,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
