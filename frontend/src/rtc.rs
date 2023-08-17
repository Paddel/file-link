use wasm_bindgen::prelude::*;

pub mod chat;

use chat::chat_model::*;

use crate::rtc::chat::web_rtc_manager::WebRTCManager;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    yew::Renderer::<ChatModel<WebRTCManager>>::new().render();
    Ok(())
}
