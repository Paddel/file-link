use std::vec::Vec;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::accept_async;

use crate::shared::{SessionDetails, SessionCode, SessionCheck, SessionCheckResult, SessionClient, SessionAnswerForward, SessionHostResult};

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
                Self::handle_ws_connection(connection, connections.clone()).await;
                println!("WebSocket connection {} opened", addr);
            }
            Err(e) => {
                eprintln!("Failed to accept WebSocket connection from {}: {}", addr, e);
            }
        }
    }

    async fn handle_ws_connection(
        connection: Arc<connection::Connection>,
        connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
    ) {
        let connection_clone = connection.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = connection_clone.read().await {
                if let tungstenite::Message::Text(text) = msg {
                    if let Some(session_details) = Self::decode_text_message(&text).await {
                        let response = match session_details {
                            SessionDetails::SessionHost(_) => Self::process_session_host(connection_clone.clone(), &session_details).await,
                            SessionDetails::SessionClient(_) => Self::process_session_client(connection_clone.clone(), connections.clone(), &session_details).await,
                        };
                        Self::send_response(connection_clone.clone(), response).await;
                    }
                }
            }

            let mut connections_lock = connections.write().await;
            let index = connections_lock.iter().position(|c| c.get_uuid() == connection_clone.get_uuid()).unwrap();
            connections_lock.remove(index);
            println!("WebSocket connection {} closed", connection_clone.get_uuid());
        });
    }
    
    async fn decode_text_message(text: &str) -> Option<SessionDetails> {
        match serde_json::from_str::<SessionDetails>(text) {
            Ok(details) => Some(details),
            Err(_) => None,
        }
    }
    
    async fn process_session_host(connection: Arc<connection::Connection>, session_details: &SessionDetails) -> String {
        let session_host = if let SessionDetails::SessionHost(host) = session_details {
            host
        } else {
            return String::new();
        };
    
        connection.set_details(session_host.clone()).await;
        let code = connection.generate_code().await;
        let data = SessionHostResult::SessionCode(SessionCode { code });
        serde_json::to_string(&data).unwrap()
    }
    
    async fn extract_session_client(session_details: &SessionDetails) -> Option<&SessionClient> {
        match session_details {
            SessionDetails::SessionClient(client) => Some(client),
            _ => None,
        }
    }

    fn extract_code_and_password(session_client: &SessionClient) -> (String, String) {
        match session_client {
            SessionClient::SessionFetchOffer(session_fetch_offer) => {
                (session_fetch_offer.code.clone(), session_fetch_offer.password.clone())
            }
            SessionClient::SessionAnswer(session_answer) => {
                (session_answer.code.clone(), session_answer.password.clone())
            }
        }
    }

    async fn find_wanted_connection(code: &str, connections: &[Arc<connection::Connection>]) -> Option<Arc<connection::Connection>> {
        for connection in connections.iter() {
            let code_other = connection.get_code().await;
            if code_other.is_some() && code_other.clone().unwrap() == code {
                return Some(connection.clone());
            }
        }
        None
    }

    async fn determine_session_result(wanted_connection: &Arc<connection::Connection>, password: &str, session_client: &SessionClient) -> SessionCheckResult {
        if let Some(pwd) = wanted_connection.get_password().await {
            if pwd == password {
                if let Some(wanted_details) = wanted_connection.get_details().await {
                    match session_client {
                        SessionClient::SessionFetchOffer(_) => {
                            SessionCheckResult::Success(wanted_details)
                        }
                        SessionClient::SessionAnswer(_) => {
                            SessionCheckResult::Success(wanted_details)
                        }
                    }
                } else {
                    SessionCheckResult::NotFound
                }
            } else {
                SessionCheckResult::WrongPassword
            }
        } else {
            SessionCheckResult::NotFound
        }
    }

    async fn handle_forwarding(connection: &Arc<connection::Connection>, session_client: &SessionClient, wanted_connection: Option<&Arc<connection::Connection>>) {
        if let SessionClient::SessionAnswer(session_answer) = session_client {
            if wanted_connection.is_some() {
                let data_forward = SessionHostResult::SessionAnswerForward(SessionAnswerForward {
                    answer: session_answer.answer.clone(),
                });
                let forward = serde_json::to_string(&data_forward).unwrap();
                if let Err(err) = wanted_connection.unwrap().write(tungstenite::Message::Text(forward.clone())).await {
                    eprintln!("Failed to send a message to {}: {}", connection.get_uuid(), err);
                }
            }
        }
    }

    async fn process_session_client(
        connection: Arc<connection::Connection>,
        connections: Arc<RwLock<Vec<Arc<connection::Connection>>>>,
        session_details: &SessionDetails,
    ) -> String {
        let connections_lock = connections.read().await;
        if let Some(session_client) = Self::extract_session_client(session_details).await {
            let (code, password) = Self::extract_code_and_password(session_client);
            let wanted_connection = Self::find_wanted_connection(&code, &connections_lock).await;
            
            let result = if let Some(connection) = &wanted_connection {
                Self::determine_session_result(connection, &password, session_client).await
            } else {
                SessionCheckResult::NotFound
            };

            Self::handle_forwarding(&connection, session_client, wanted_connection.as_ref()).await;

            let data = SessionCheck { result };
            serde_json::to_string(&data).unwrap()
        } else {
            String::new()
        }
    }

    
    async fn send_response(connection: Arc<connection::Connection>, response: String) {
        if let Err(err) = connection.write(tungstenite::Message::Text(response.clone())).await {
            eprintln!("Failed to send a message to {}: {}", connection.get_uuid(), err);
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
}