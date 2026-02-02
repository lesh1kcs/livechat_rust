mod auth;
mod database;

use axum::{
    extract::WebSocketUpgrade,
    extract::Extension,
    extract::ws::{Message, WebSocket},
    response::{IntoResponse, Html, Redirect},
    routing::{get,post},
    Router,
};

use tower_http::{services::ServeDir}; 
use std::{net::SocketAddr};
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use database::ChatDatabase;

use crate::auth::login_handler;
use crate::auth::auth_get;

type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

async fn websocket_handler(ws:WebSocketUpgrade, Extension(clients): Extension<Clients>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

async fn handle_socket(socket: WebSocket, clients: Clients) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    clients.lock().unwrap().push(tx);
    println!("New client connected. Total clients: {}", clients.lock().unwrap().len());

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let clients_clone = clients.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await{
            match msg {
                Message::Text(text) => {
                    println!("Received: {}", text);
                    mpsc_message(&clients_clone, Message::Text(text)).await;
                },
                Message::Close(_) => {
                    println!("Client disconnected");
                    break;
                }
                _ => {}
            }
        }
    });
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}
async fn mpsc_message(clients: &Clients, message: Message) {
    let mut clients_lock = clients.lock().unwrap();
    let mut disconnected = Vec::new();

    for (i, client) in clients_lock.iter().enumerate() {
        if client.send(message.clone()).is_err() {
            disconnected.push(i);
        }
    }

    for &i in disconnected.iter().rev() {
        clients_lock.remove(i);
    }
}

async fn index_handler() -> impl IntoResponse {
    match std::fs::read_to_string("templates/index.html") {
        Ok(content) => Html(content),
        Err(e) => {
            eprintln!("Failed to read index.html: {}", e);
            Html("<h1>Error loading chat page</h1>".to_string())
        }
    }
}

// async fn authentication_handler() -> impl IntoResponse {
//     Html(std::fs::read_to_string("templates/authentication.html").unwrap())
// }

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let db = ChatDatabase::new("chat.db").expect("Failed to initialize database!");
    
    let app = Router::new()
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
