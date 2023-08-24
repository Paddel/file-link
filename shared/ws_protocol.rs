/*
    shared/ws_protocol.rs

    This module defines the WebSocket protocol data structures used for communication between the frontend and backend.
*/

/* Frontend -> Backend */
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionDetails {
    SessionHost(SessionHost),
    SessionClient(SessionClient),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionHost {
    mode: String,
    offer: String,
    password: String,
    compression: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionClient {
    code: String,
    password: String,
}

/* Backend -> Frontend */
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionCode {
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCheck {
    pub result: SessionCheckResult,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SessionCheckResult {
    Success,
    WrongPassword,
    SessionNotFound,
}