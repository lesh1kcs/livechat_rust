use axum::{
    extract::WebSocketUpgrade,
    extract::ws::{Message, WebSocket},
    extract::Extension,
    response::{IntoResponse, Html},
};
use futures::{SinkExt, StreamExt};
use std::sync::{Arc,Mutex};
use tokio::sync::mpsc;

pub type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

pub async fn websocket_handler(ws:WebSocketUpgrade, Extension(clients): Extension<Clients>
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

pub async fn index_handler() -> impl IntoResponse {
    match std::fs::read_to_string("templates/index.html") {
        Ok(content) => Html(content),
        Err(e) => {
            eprintln!("Failed to read index.html: {}", e);
            Html("<h1>Error loading chat page</h1>".to_string())
        }
    }
}