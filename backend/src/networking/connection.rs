use futures_core::Stream;
use futures_util::{Sink, SinkExt, stream::StreamExt};
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message, tungstenite::Error as TungsteniteError};
use uuid::Uuid;

pub struct Connection {
    uuid: Uuid,
    reader: RwLock<Pin<Box<dyn Stream<Item = Result<Message, TungsteniteError>> + Send + Sync>>>,
    writer: RwLock<Pin<Box<dyn Sink<Message, Error = TungsteniteError> + Send + Sync>>>,
}

impl Connection {
    pub fn new(ws_stream: WebSocketStream<TcpStream>) -> Self {
        let (writer, reader) = ws_stream.split();
        Self {
            uuid: Uuid::new_v4(),
            reader: RwLock::new(Box::pin(reader)),
            writer: RwLock::new(Box::pin(writer)),
        }
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
