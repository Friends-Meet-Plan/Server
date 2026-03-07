use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE events RENAME COLUMN memory_image TO memory_image_base64;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE events ALTER COLUMN memory_image_base64 TYPE text;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE events ALTER COLUMN memory_image_base64 TYPE varchar;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE events RENAME COLUMN memory_image_base64 TO memory_image;",
            )
            .await?;

        Ok(())
    }
}
