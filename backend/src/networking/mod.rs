use std::vec::Vec;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;

use crate::shared::SessionDetails;

mod connection;

pub struct NetworkManager {
    connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            connections: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_signaling_server(&self) {
        let addr = "0.0.0.0:9000";
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        println!("WebSocket server started at ws://{}", addr);

        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            tokio::spawn(Self::handle_connection(
                stream,
                self.connections.clone(),
            ));
        }
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
    ) {
        let addr = stream.peer_addr().unwrap();
    
        match accept_async(stream).await {
            Ok(ws_stream) => {
                println!("New WebSocket connection: {}", addr);
                let mut connections_lock = connections.write().await;
                let connection = Arc::new(connection::Connection::new(ws_stream));
                connections_lock.push(connection.clone());
                Self::handle_ws_connection(connection).await;
            }
            Err(e) => {
                eprintln!("Failed to accept WebSocket connection from {}: {}", addr, e);
            }
        }
    }

    async fn handle_ws_connection(
        connection: Arc<connection::Connection>,
    ) {
        let connection_clone = connection.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = connection_clone.read().await {
                if let tungstenite::Message::Text(text) = msg {
                    let session_details: Result<SessionDetails, serde_json::Error> = serde_json::from_str(&text);
                    if session_details.is_err() {
                        continue;
                    }
                    
                    println!("Received text message {}", text);
                    let response = connection_clone.session_execute(session_details.unwrap()).await;
                    if let Err(err) = connection_clone.write(tungstenite::Message::Text(response.clone())).await {
                        eprintln!("Failed to send a message to {}: {}", connection_clone.get_uuid(), err);
                    }
                    println!("Rsponds with {}", response);
                }
            }
        });
    }

    pub fn initialize_networking() -> Arc<NetworkManager> {
        let network_manager = Arc::new(NetworkManager::new());
        let network_manager_clone = network_manager.clone();
        tokio::spawn(async move {
            network_manager_clone.start_signaling_server().await;
        });
        network_manager
    }
}