use std::collections::HashMap;

use gloo::net::http::{Request, RequestBuilder};
use wasm_bindgen::JsValue;
use web_sys::{console, RequestMode};

use crate::shared::SessionCreate;
use crate::constants::{HOST_ADDRESS, PORT};

pub struct ApiService {}

impl ApiService {
    pub fn new() -> ApiService {
        ApiService {}
    }

    pub fn post_session(callback: impl FnOnce(), password: &str, compression_level: i32) {
        let session_create = SessionCreate {
            compression_level: compression_level as u8,
            password: "ab".to_string(),
        };
        let session_create_str = serde_json::to_string(&session_create).expect("Serialization failed");
        let url = Self::get_host_address() + "/api/sessions";
        console::log_1(&JsValue::from_str(&format!("session_create_str: {}", session_create_str)));
        let request = Request::post(&url).json(&session_create_str);
        ApiService::execute_api_call(callback, request);
    }

    fn execute_api_call(callback: impl FnOnce(), request: Result<Request, gloo::net::Error>) {
        if request.is_err() {
            console::log_1(&JsValue::from_str(&format!("Error: {:?}", request.err())));
            return
        }

        let request = request.unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            // let result = request.mode(RequestMode::NoCors).send().await;
            let result = request.send().await;

            if result.is_err() {
                console::log_1(&JsValue::from_str(&format!("Error: {:?}", result.err())));
                return
            }
            
            let result = result.unwrap();
            console::log_1(&JsValue::from_str(&format!("Result: {:?}", result)));

        });
    }

    fn get_host_address() -> String {
        let address = "http://".to_string() + HOST_ADDRESS + ":" + &PORT.to_string();
        address
    }
}
