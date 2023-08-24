use yew::Callback;
use yew_websocket::websocket::{WebSocketTask, WebSocketService, WebSocketStatus};

#[derive(Clone, Debug, PartialEq)]
pub enum WebsocketMessage {

}

pub struct WebsocketManager {
    websocket_manager: Option<WebSocketTask>,
    callback: Callback<WebsocketMessage>,
}

impl WebsocketManager {
    pub fn new(url: &str, callback: Callback<WebsocketMessage>) -> Self {

        let callback_inner = ctx.link().callback(|Json(data)| Msg::WsReady(data));
        let notification: Callback<WebSocketStatus> = ctx.link().batch_callback(move |status| match status {
            WebSocketStatus::Opened => {Some(Msg::WsOpened().into())},
            WebSocketStatus::Closed | WebSocketStatus::Error => {
                console::log_1(&format!("ws close: {:?}", status).into());
                Some(Msg::WsLost.into())
            }
        });
        let task = match WebSocketService::connect(
            WEBSOCKET_ADDRESS,
            callback,
            notification,
        ) {
            Ok(task) => Some(task),
            Err(_) => None,
        };


        let websocket_manager = WebSocketService::connect_binary(url, on_message.clone(), Callback::from(|_| ())).unwrap();
        Self {
            websocket_manager,
            on_message,
        }
    }

    pub fn send(&mut self, message: WebsocketMessage) {
        self.ws.send_binary(Ok(message));
    }
}