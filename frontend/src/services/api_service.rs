use std::collections::HashMap;

use reqwest::{Request, Method, RequestBuilder};

use crate::shared::FRONTEND_CONFIG;

pub struct ApiService {}

impl ApiService {
    pub fn new() -> ApiService {
        ApiService {}
    }

    pub fn post_session(callback: impl FnOnce(), password: &str, compression_level: i32) {
        let url = Self::get_host_address() + "/api/sessions";

        let mut data = HashMap::new();
        data.insert("password", password);
        let compression_level_str = compression_level.to_string();
        data.insert("compression_level", &compression_level_str);

        let client = reqwest::Client::new();
        let request = client.post(url).json(&data);
        Self::execute_api_call(callback, request);
    }

    fn execute_api_call(callback: impl FnOnce(), request: RequestBuilder) {
        tokio::spawn(async move {
            let resp = request.send().await?;
            // callback();
            resp.bytes().await
            // callback();
        });
    }

    fn get_host_address() -> String {
        let frontend_config = &*FRONTEND_CONFIG;
        frontend_config.api_address.clone()
    }
}
