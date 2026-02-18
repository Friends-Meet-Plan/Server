use dotenvy::dotenv;
use axum::Router;
use sea_orm_migration::MigratorTrait;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::controllers::{auth_controller, friendship_controller};
use crate::migration::Migrator;
use controllers::users_controller;
use crate::api_doc::api_doc::ApiDoc;

mod auth;
mod controllers;
mod db;
mod entities;
mod migration;
mod api_doc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_connection = db::init_db()
        .await
        .expect("db connection failed");

    Migrator::up(&db_connection, None)
        .await
        .expect("migration failed");

    let app_router = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .merge(auth_controller::router())
        .merge(users_controller::router())
        .merge(friendship_controller::router());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Starts on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app_router.with_state(db_connection))
        .await
        .unwrap();
}
