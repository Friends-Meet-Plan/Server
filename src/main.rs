use dotenvy::dotenv;

use axum::Router;

use sea_orm_migration::MigratorTrait;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod auth;
mod controllers;
mod db;
mod entities;
mod migration;

use crate::controllers::auth_controller;
use crate::migration::Migrator;
use controllers::users_controller;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_connection = db::init_db().await.expect("db connection failed");

    Migrator::up(&db_connection, None)
        .await
        .expect("migration failed");

    let app_router = Router::new()
        .merge(auth_controller::router())
        .merge(users_controller::router());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Starts on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app_router.with_state(db_connection))
        .await
        .unwrap();
}
