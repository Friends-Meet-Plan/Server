use dotenvy::dotenv;

use axum::{
    routing::get,
    Router,
};

use std::net::SocketAddr;
use tokio::net::TcpListener;

mod auth;
use auth::{middleware};
use middleware::{AuthUser};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .route("/base", get(base_route));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Starts on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn base_route(user: AuthUser) -> String {
    format!("Hello user {}", user.user_id)
}
