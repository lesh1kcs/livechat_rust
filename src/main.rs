use axum::{
    extract::WebSocketUpgrade,
    extract::ws::{Message, WebSocket},
    response::{IntoResponse, Html},
    routing::get,
    Router,
};

use tower_http::services::ServeDir; 
use std::net::SocketAddr;
use futures::{channel::mpsc, SinkExt, StreamExt};
use std::sync::{Arc, Mutex};

type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

async fn websocket_handler(ws:WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) =>{
                println!("Got message from client: {}", text);
                let formatted_response =  format!("{}", text);
                socket.send(Message::Text(formatted_response)).await.unwrap();
            },
            Message::Close(_) =>{
                println!("Client disconnected");
                break;
            },
            _ => {}

        }
        
    }
}

async fn index_handler() -> impl IntoResponse {
    Html(std::fs::read_to_string("templates/index.html").unwrap())
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/send", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"));

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
