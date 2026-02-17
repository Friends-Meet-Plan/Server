use dotenvy::dotenv;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;
use serde::Serialize;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set};
use sea_orm_migration::MigratorTrait;

mod auth;
mod db;
mod entities;
mod migration;

use crate::entities::{User, UserActiveModel};
use crate::migration::Migrator;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let db_connection = db::init_db()
        .await
        .expect("db connection failed");
    
    Migrator::up(&db_connection, None)
        .await
        .expect("migration failed");
    
    let app_router = Router::new();
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Starts on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app_router.with_state(db_connection)
    )
    .await
    .unwrap();
}