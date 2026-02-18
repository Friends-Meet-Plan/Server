use sea_orm_migration::prelude::*;

mod m0001_create_users;
mod m0002_create_friendships;

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
        ]
    }
}
