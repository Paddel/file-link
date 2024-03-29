use gloo::net::http::Request;
use wasm_bindgen::JsValue;
use yew::Callback;

use crate::constants::{HOST_ADDRESS, PORT};
use crate::shared::{
    ClientGetDetails, ClientGetDetailsResult, ClientJoin, ClientJoinResult, HostCreate, HostCreateResult, HostPollResult,
};

const POLL_WAIT_TIME_ONE_TIMOUT: u64 = 1000;

pub enum ApiServiceMessage {
    HostCreate(Result<HostCreateResult, u16>),
    HostPoll(Result<HostPollResult, u16>),
    ClientDetails(Result<ClientGetDetailsResult, u16>),
    ClientJoin(Result<ClientJoinResult, u16>),
}

pub mod api_service {
    use std::time::Duration;
    use web_sys::console;

    use super::*;

    pub fn create_session(
        callback: Callback<ApiServiceMessage>,
        connection_details: String,
        password: String,
        compression_level: u8,
    ) {
        let session_create = HostCreate {
            connection_details,
            compression_level,
            password,
        };
        let session_create_str =
            serde_json::to_string(&session_create).expect("Serialization failed");
        let url = get_host_address() + "/api/sessions";
        let request = Request::post(&url).json(&session_create_str);

        let callback_result = move |response: Result<String, u16>| {
            if response.is_err() {
                let status = response.unwrap_err();
                callback.emit(ApiServiceMessage::HostCreate(Err(status)));
                return;
            }

            let response = response.unwrap();
            let response = serde_json::from_str::<HostCreateResult>(&response);
            if response.is_err() {
                console::log_1(&JsValue::from_str(&format!(
                    "Error creating session: {:?}",
                    response.err()
                )));
                return;
            }
            let response = response.unwrap();
            callback.emit(ApiServiceMessage::HostCreate(Ok(response)));
        };

        if request.is_err() {
            console::log_1(&JsValue::from_str(&format!("Error: {:?}", request.err())));
            return;
        }

        execute_api_call(callback_result, request.unwrap());
    }

    pub fn poll_session(callback: Callback<ApiServiceMessage>, code: String) {
        let url = get_host_address() + "/api/sessions/poll/" + &code;

        let callback_result = move |response: Result<String, u16>| {
            if response.is_err() {
                console::log_1(&JsValue::from_str(&format!("Error: {:?}", response.err())));
                return;
            }

            let response = response.unwrap();
            let response: Result<_, serde_json::Error> = serde_json::from_str::<HostPollResult>(&response);
            if response.is_err() {
                console::log_1(&JsValue::from_str(&format!(
                    "Error joining session: {:?}",
                    response.err()
                )));
                return;
            }
            let response = response.unwrap();
            callback.emit(ApiServiceMessage::HostPoll(Ok(response)));
        };

        let request_builder = move |url: &str| Request::get(url).build().expect("Request build failed");
        execute_api_poll(callback_result, url, request_builder);
    }

    pub fn get_session_details(
        callback: Callback<ApiServiceMessage>,
        code: &str,
        password: Option<String>,
    ) {
        let session_join = ClientGetDetails {
            code: code.to_string(),
            password: password.unwrap_or("".to_string()),
        };
        let session_join_str = serde_json::to_string(&session_join).expect("Serialization failed");
        let url = get_host_address() + "/api/sessions/details";
        let request = Request::post(&url).json(&session_join_str);

        let callback_result = move |response: Result<String, u16>| {
            if response.is_err() {
                let status = response.unwrap_err();
                callback.emit(ApiServiceMessage::ClientDetails(Err(status)));
                return;
            }

            let response = response.unwrap();
            let response = serde_json::from_str::<ClientGetDetailsResult>(&response);
            if response.is_err() {
                console::log_1(&JsValue::from_str(&format!(
                    "Error joining session: {:?}",
                    response.err()
                )));
                return;
            }
            let response = response.unwrap();
            callback.emit(ApiServiceMessage::ClientDetails(Ok(response)));
        };

        if request.is_err() {
            console::log_1(&JsValue::from_str(&format!("Error: {:?}", request.err())));
            return;
        }

        execute_api_call(callback_result, request.unwrap());
    }

    pub fn join_session(
        callback: Callback<ApiServiceMessage>,
        code: String,
        password: Option<String>,
        connection_details: String,
    ) {
        let session_join = ClientJoin {
            code,
            password: password.unwrap_or("".to_string()),
            connection_details,
        };
        let session_join_str = serde_json::to_string(&session_join).expect("Serialization failed");
        let url = get_host_address() + "/api/sessions/join";
        let request = Request::post(&url).json(&session_join_str);

        let callback_result = move |response: Result<String, u16>| {
            if response.is_err() {
                let status = response.unwrap_err();
                callback.emit(ApiServiceMessage::ClientJoin(Err(status)));
                return;
            }

            let response = response.unwrap();
            let response = serde_json::from_str::<ClientJoinResult>(&response);
            if response.is_err() {
                console::log_1(&JsValue::from_str(&format!(
                    "Error joining session: {:?}",
                    response.err()
                )));
                return;
            }
            let response = response.unwrap();
            callback.emit(ApiServiceMessage::ClientJoin(Ok(response)));
        };

        if request.is_err() {
            console::log_1(&JsValue::from_str(&format!("Error: {:?}", request.err())));
            return;
        }

        execute_api_call(callback_result, request.unwrap());
    }

    fn execute_api_call(callback: impl FnOnce(Result<String, u16>) + 'static, request: Request) {
        wasm_bindgen_futures::spawn_local(async move {
            let response = request.send().await;
            
            if response.is_err() {
                return callback(Err(500));
            }
            
            let response = response.unwrap();
            
            if response.status() != 200 {
                let status = response.status();
                return callback(Err(status));
            }
            let response = response.text().await;
            if response.is_err() {
                return callback(Err(500));
            }
            callback(Ok(response.unwrap()));
        });
    }

    fn execute_api_poll(callback: impl FnOnce(Result<String, u16>) + Clone + 'static, url: String, request_builder: impl FnOnce(&str) -> Request + std::marker::Copy + 'static) {
        wasm_bindgen_futures::spawn_local(async move {
            let request = request_builder(&url);
            let response = request.send().await;
            
            if response.is_err() {
                return callback(Err(500));
            }
            
            let response = response.unwrap();

            if response.status() == 502 || response.status() == 408 {
                async_std::task::sleep(Duration::from_millis(POLL_WAIT_TIME_ONE_TIMOUT)).await;
                execute_api_poll(callback.clone(), url, request_builder);
                return;
            }
            
            if response.status() != 200 {
                let status = response.status();
                return callback(Err(status));
            }
            let response = response.text().await;
            if response.is_err() {
                return callback(Err(500));
            }
            callback(Ok(response.unwrap()));
        });
    }

    fn get_host_address() -> String {
        let address = HOST_ADDRESS.to_string() + ":" + &PORT.to_string();
        address
    }
}
