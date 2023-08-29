// This code is based on the example from the security-union's yew-websocket repository:
// https://github.com/security-union/yew-websocket/blob/master/src/websocket.rs


use gloo::events::EventListener;
use yew::Callback;
use wasm_bindgen::JsCast;
use web_sys::{BinaryType, Event, MessageEvent, WebSocket};

#[derive(Clone, Debug, PartialEq)]
pub enum WebSocketMessage {
    Text(String),
    Open,
    Close,
    Err,
}

pub struct WsConnection {
    ws: Option<WebSocket>,
    callback: Option<Callback<WebSocketMessage>>,
    #[allow(dead_code)]
    event_listeners: Vec<EventListener>,
}

impl WsConnection {
    pub fn new(url: &str, callback: Callback<WebSocketMessage>) -> Result<Self, anyhow::Error> {
        let ws = WebSocket::new(url).map_err(|err| {
            let js_error = err.unchecked_into::<js_sys::Error>().to_string().as_string().unwrap();
            anyhow::Error::msg(js_error)
        })?;

        ws.set_binary_type(BinaryType::Arraybuffer);

        let mut event_listeners = Vec::new();

        {
            let callback = callback.clone();
            let on_open = EventListener::new(&ws, "open", move |_| {
                callback.emit(WebSocketMessage::Open);
            });
            event_listeners.push(on_open);
        }

        {
            let callback = callback.clone();
            let on_close = EventListener::new(&ws, "close", move |_| {
                callback.emit(WebSocketMessage::Close);
            });
            event_listeners.push(on_close);
        }

        {
            let callback = callback.clone();
            let on_error = EventListener::new(&ws, "error", move |_| {
                callback.emit(WebSocketMessage::Err);
            });
            event_listeners.push(on_error);
        }

        {
            let callback = callback.clone();
            let on_message = EventListener::new(&ws, "message", move |event: &Event| {
                if let Some(msg_event) = event.dyn_ref::<MessageEvent>() {
                    if let Some(text) = msg_event.data().as_string() {
                        callback.emit(WebSocketMessage::Text(text));
                    }
                }
            });
            event_listeners.push(on_message);
        }

        Ok(WsConnection {
            ws: Some(ws),
            callback: Some(callback),
            event_listeners,
        })
    }

    pub fn send_text(&mut self, text: &str) {
        if let Some(ws) = &self.ws {
            if ws.send_with_str(&text).is_err() {
                if let Some(cb) = &self.callback {
                    cb.emit(WebSocketMessage::Err);
                }
            }
        }
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        if let Some(ws) = &self.ws {
            if matches!(ws.ready_state(), WebSocket::CONNECTING | WebSocket::OPEN) {
                let _ = ws.close();
            }
        }
    }
}
