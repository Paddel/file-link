use futures_core::Stream;
use futures_util::{Sink, SinkExt, stream::StreamExt};
use std::pin::Pin;

use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message, tungstenite::Error as TungsteniteError};
use uuid::Uuid;

use crate::shared::{SessionDetails, SessionHost};

struct ConnectionProperties {
    details: Option<SessionHost>,
    code: Option<String>,
}

pub struct Connection {
    uuid: Uuid,
    reader: RwLock<Pin<Box<dyn Stream<Item = Result<Message, TungsteniteError>> + Send + Sync>>>,
    writer: RwLock<Pin<Box<dyn Sink<Message, Error = TungsteniteError> + Send + Sync>>>,
    properties: RwLock<ConnectionProperties>,
}

impl Connection {
    pub fn new(ws_stream: WebSocketStream<TcpStream>) -> Self {
        let properties = ConnectionProperties {
            details: None,
            code: None,
        };
        
        let (writer, reader) = ws_stream.split();
        Self {
            uuid: Uuid::new_v4(),
            reader: RwLock::new(Box::pin(reader)),
            writer: RwLock::new(Box::pin(writer)),
            properties: RwLock::new(properties),
        }
    }

    pub async fn generate_code(&self) -> String {
        let mut properties = self.properties.write().await;
        let code: String = (0..10)
        .map(|_| rand::thread_rng().gen_range('a'..='z'))
        .chain((0..10).map(|_| rand::thread_rng().gen_range('0'..='9')))
        .collect();
        properties.code = Some(code.clone());
        code
    }

    pub async fn get_code(&self) -> Option<String> {
        let properties = self.properties.read().await;
        if properties.code.is_none() {
            println!("code is none in get_code");
        }
        properties.code.clone()
    }

    pub async fn get_password(&self) -> Option<String> {
        let properties = self.properties.read().await;
        if properties.details.is_none() {
            println!("details is none in get_password");
        }
        properties.details.as_ref().map(|details| details.password.clone())
    }
    
    pub async fn get_details(&self) -> Option<SessionHost> {
        let properties = self.properties.read().await;
        if properties.details.is_none() {
            println!("details is none in get_details");
        }
        properties.details.as_ref().cloned()
    }

    pub async fn set_details(&self, details: SessionHost) {
        let mut properties = self.properties.write().await;
        properties.details = Some(details);
    }


    // pub async fn session_execute(&self, details: SessionDetails) -> String {
    //     let mut properties = self.properties.write().await;
    //     properties.details = Some(details.clone());

    //     match details {
    //         SessionDetails::SessionHost(_) => {
    //             properties.code = Some(Self::generate_code());
    //             let data = SessionCode {
    //                 code: properties.code.clone().unwrap(),
    //             };
    //             serde_json::to_string(&data).unwrap()
    //         }
    //         SessionDetails::SessionClient(_) => {
    //             let data = SessionCheck {
    //                 result: SessionCheckResult::NotFound,
    //             };
    //             serde_json::to_string(&data).unwrap()
    //         }
    //     }
    // }

    pub async fn read(&self) -> Option<Result<Message, TungsteniteError>> {
        self.reader.write().await.next().await
    }

    pub async fn write(&self, message: Message) -> Result<(), TungsteniteError> {
        self.writer.write().await.send(message).await
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
