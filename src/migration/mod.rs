use sea_orm_migration::prelude::*;

mod m0001_create_users;
mod m0002_create_friendships;
mod m0003_create_busydays;
mod m0004_create_invitations;
mod m0005_create_invitation_dates;
mod m0006_create_events;
mod m0007_create_event_participants;
mod m0008_create_wish_places;
mod m0009_add_events_wish_place_fk;
mod m0010_refactor_invitations_event_link;
mod m0011_user_events_drop_invitations;
mod m0012_drop_event_participants;
mod m0013_event_finished_memory_image;
mod m0014_event_memory_image_base64;

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
            Box::new(m0006_create_events::Migration),
            Box::new(m0007_create_event_participants::Migration),
            Box::new(m0008_create_wish_places::Migration),
            Box::new(m0009_add_events_wish_place_fk::Migration),
            Box::new(m0010_refactor_invitations_event_link::Migration),
            Box::new(m0011_user_events_drop_invitations::Migration),
            Box::new(m0012_drop_event_participants::Migration),
            Box::new(m0013_event_finished_memory_image::Migration),
            Box::new(m0014_event_memory_image_base64::Migration),
        ]
    }
}
