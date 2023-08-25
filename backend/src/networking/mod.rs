use std::vec::Vec;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
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
                    let session_details: Result<SessionDetails, serde_json::Error> = serde_json::from_str(&text);
                    println!("Received text message {}", text);
                    if session_details.is_err() {
                        continue;
                    }

                    let session_details = session_details.unwrap();
                    let response = match session_details.clone() {
                        SessionDetails::SessionHost(session_host) => {
                            connection_clone.set_details(session_host).await;
                            let code = connection_clone.generate_code().await;
                            let data = SessionHostResult::SessionCode(SessionCode {
                                code,
                            });
                            serde_json::to_string(&data).unwrap()
                        }
                        SessionDetails::SessionClient(session_client) => {
                            let connections_lock = connections.read().await;
                            let mut wanted_connection = None;

                            let (code, password) = match session_client.clone() {
                                SessionClient::SessionFetchOffer(session_fetch_offer) => {
                                    (session_fetch_offer.code, session_fetch_offer.password)
                                }
                                SessionClient::SessionAnswer(session_answer) => {
                                    (session_answer.code, session_answer.password)
                                }
                            };
                            
                            for connection in connections_lock.iter() {
                                let code_other = connection.get_code().await;
                                if code_other.is_some() && code_other.clone().unwrap() == code {
                                    wanted_connection = Some(connection);
                                    break;
                                }
                            }

                            let result = if let Some(wanted_connection) = wanted_connection {
                                if let Some(pwd) = wanted_connection.get_password().await {
                                    if pwd == password {
                                        if let Some(details) = wanted_connection.get_details().await {
                                            match session_client.clone() {
                                                SessionClient::SessionFetchOffer(details_offer) => {
                                                    println!("Fetched offer from {}", details_offer.code);
                                                    SessionCheckResult::Success(details)
                                                }
                                                SessionClient::SessionAnswer(session_answer) => {
                                                    // wanted_connection.set_answer(session_answer.answer).await;
                                                    println!("Answered with {}", session_answer.answer);
                                                    SessionCheckResult::Success(details)
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
                            } else {
                                SessionCheckResult::NotFound
                            };        

                            let data = match session_client.clone() {
                                SessionClient::SessionFetchOffer(_) => {
                                    SessionCheck { result }
                                }
                                SessionClient::SessionAnswer(session_answer) => {
                                    if wanted_connection.is_some() {
                                        let wanted_connection = wanted_connection.unwrap();
                                        let data_foward = SessionHostResult::SessionAnswerForward(SessionAnswerForward {
                                            answer: session_answer.answer.clone(),
                                        });
                                        let forward = serde_json::to_string(&data_foward).unwrap();
                                        if let Err(err) = wanted_connection.write(tungstenite::Message::Text(forward.clone())).await {
                                            eprintln!("Failed to send a message to {}: {}", connection_clone.get_uuid(), err);
                                        }
                                        println!("Forwarded answer to {}", session_answer.code);
                                    }

                                    SessionCheck { result }
                                }
                            };                    
                            
                            serde_json::to_string(&data).unwrap()                            
                        }
                    };

                    //remove connection if it is a client
                    // if let SessionDetails::SessionClient(_) = session_details.clone() {
                    //     let mut connections_lock = connections.write().await;
                    //     let mut index = None;
                    //     for (i, connection) in connections_lock.iter().enumerate() {
                    //         if connection.get_uuid() == connection_clone.get_uuid() {
                    //             index = Some(i);
                    //             break;
                    //         }
                    //     }
                    //     if index.is_some() {
                    //         connections_lock.remove(index.unwrap());
                    //         println!("Removed connection {}", connection_clone.get_uuid());
                    //     }
                    //     else {
                    //         println!("Failed to remove connection {}", connection_clone.get_uuid());
                    //     }
                    // }
                    
                    // println!("Received text message {}", text);
                    // let response = connection_clone.session_execute(session_details.unwrap()).await;
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