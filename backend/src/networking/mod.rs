use std::vec::Vec;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::accept_async;

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
        let addr = "127.0.0.1:9000";
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        println!("WebSocket server started at ws://{}", addr);

        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            tokio::spawn(handle_connection(
                stream,
                self.connections.clone(),
            ));
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
) {
    let addr = stream.peer_addr().unwrap();

    match accept_async(stream).await {
        Ok(ws_stream) => {
            let mut connections_lock = connections.write().await;
            let connection = Arc::new(connection::Connection::new(ws_stream));
            connections_lock.push(connection.clone());
            handle_ws_connection(connection, connections.clone());
        }
        Err(e) => {
            eprintln!("Failed to accept WebSocket connection from {}: {}", addr, e);
        }
    }
}

fn handle_ws_connection(
    connection: Arc<connection::Connection>,
    connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
) {
    tokio::spawn(async move {
        while let Some(Ok(msg)) = connection.read().await {
            message_broadcast(connection.clone(), connections.clone(), msg).await;
        }
    });
}

async fn message_broadcast(
    sender: Arc<connection::Connection>,
    connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
    msg: tungstenite::Message,
) {
    let connections_lock = connections.read().await;
    for connection in connections_lock.iter().filter(|&conn| conn.get_uuid() != sender.get_uuid()) {
        if let Err(err) = connection.write(msg.clone()).await {
            eprintln!("Failed to send a message to {}: {}", connection.get_uuid(), err);
        }
    }
}

pub fn initialize_networking() -> Arc<NetworkManager> {
    let network_manager = Arc::new(NetworkManager::new());
    let network_manager_clone = network_manager.clone();
    tokio::spawn(async move {
        network_manager_clone.start_signaling_server().await;
    });
    network_manager
}