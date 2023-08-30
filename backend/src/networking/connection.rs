use futures_core::Stream;
use futures_util::{Sink, SinkExt, stream::StreamExt};
use std::pin::Pin;

use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message, tungstenite::Error as TungsteniteError};
use uuid::Uuid;

use crate::shared::SessionHost;

const CODE_CHAR_SET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

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
        let code: String = (0..10)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CODE_CHAR_SET.len());
                CODE_CHAR_SET.chars().nth(idx).unwrap()
            })
            .collect();
        let mut properties = self.properties.write().await;
        properties.code = Some(code.clone());
        code
    }
    
    pub async fn get_code(&self) -> Option<String> {
        let properties = self.properties.read().await;
        properties.code.clone()
    }

    pub async fn get_password(&self) -> Option<String> {
        let properties = self.properties.read().await;
        properties.details.as_ref().map(|details| details.password.clone())
    }
    
    pub async fn get_details(&self) -> Option<SessionHost> {
        let properties = self.properties.read().await;
        properties.details.as_ref().cloned()
    }

    pub async fn set_details(&self, details: SessionHost) {
        let mut properties = self.properties.write().await;
        properties.details = Some(details);
    }

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
