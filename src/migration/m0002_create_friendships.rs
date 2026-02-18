use crate::migration::uuid_pk;
use sea_orm_migration::prelude::*;
use sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DO $$\nBEGIN\n    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'friendship_status') THEN\n        CREATE TYPE friendship_status AS ENUM ('pending', 'accepted');\n    END IF;\nEND$$;"
                    .to_string(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Friendships::Table)
                    .if_not_exists()
                    .col(uuid_pk())
                    .col(ColumnDef::new(Friendships::UserId).uuid().not_null())
                    .col(ColumnDef::new(Friendships::FriendId).uuid().not_null())
                    .col(
                        ColumnDef::new(Friendships::Status)
                            .enumeration(
                                FriendshipStatus::Table,
                                [FriendshipStatus::Pending, FriendshipStatus::Accepted],
                            )
                            .not_null()
                            .default(FriendshipStatus::Pending.to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_friendships_user_id")
                            .from(Friendships::Table, Friendships::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_friendships_friend_id")
                            .from(Friendships::Table, Friendships::FriendId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_friendships_user_friend_unique")
                    .table(Friendships::Table)
                    .col(Friendships::UserId)
                    .col(Friendships::FriendId)
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
                    .name("idx_friendships_user_friend_unique")
                    .table(Friendships::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Friendships::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE IF EXISTS friendship_status".to_string(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Friendships {
    Table,
    Id,
    UserId,
    FriendId,
    Status,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum FriendshipStatus {
    #[iden = "friendship_status"]
    Table,
    #[iden = "pending"]
    Pending,
    #[iden = "accepted"]
    Accepted,
}
