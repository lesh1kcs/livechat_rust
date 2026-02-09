mod auth;
mod database;
mod handlers;

use axum::{
    extract::Extension,
    extract::ws::Message,
    response::Redirect,
};

use axum::routing::get;
use tower_http::{services::ServeDir}; 
use std::{net::SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use database::ChatDatabase;
use auth::{login_handler, auth_get};
use handlers::{websocket_handler, index_handler};

type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let db = ChatDatabase::new("chat.db").expect("Failed to initialize database!");
    
    let app = axum::Router::new()
        .route("/", get(|| async { Redirect::to("/auth") }))
        .route("/auth", get(auth_get).post(login_handler))
        .route("/api/user", get(auth::get_user_handler))
        .route("/chat", get(index_handler))
        .route("/send", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(Extension(clients))
        .layer(Extension(db));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("Listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr)
        .await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
