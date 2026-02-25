use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_events_wish_place_id")
                            .from_tbl(Events::Table)
                            .from_col(Events::WishPlaceId)
                            .to_tbl(WishPlaces::Table)
                            .to_col(WishPlaces::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .drop_foreign_key(Alias::new("fk_events_wish_place_id"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Events {
    Table,
    WishPlaceId,
}

#[derive(Iden)]
enum WishPlaces {
    Table,
    Id,
}
