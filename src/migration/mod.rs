use sea_orm_migration::prelude::*;

mod m0001_create_users;
mod m0002_create_friendships;
mod m0003_create_busydays;
mod m0004_create_invitations;
mod m0005_create_invitation_dates;

pub fn uuid_pk() -> ColumnDef {
    ColumnDef::new(Alias::new("id"))
        .uuid()
        .not_null()
        .primary_key()
        .default(Expr::cust("gen_random_uuid()"))
        .to_owned()
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m0001_create_users::Migration),
            Box::new(m0002_create_friendships::Migration),
            Box::new(m0003_create_busydays::Migration),
            Box::new(m0004_create_invitations::Migration),
            Box::new(m0005_create_invitation_dates::Migration),
        ]
    }
}
