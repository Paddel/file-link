/*
    ws_protocol.rs

    This module defines the WebSocket protocol data structures used for communication between the frontend and backend.
*/

/* Frontend -> Backend */
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type_session")]
pub enum SessionDetails {
    SessionHost(SessionHost),
    SessionClient(SessionClient),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionHost {
    offer: String,
    compression: u8,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type_client")]
pub enum SessionClient {
    SessionFetchOffer(SessionFetchOffer),
    SessionAnswer(SessionAnswer),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionFetchOffer {
    pub code: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionAnswer {
    pub code: String,
    pub password: String,
    pub answer: String,
}

/* Backend -> Frontend */
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type_host")]
pub enum SessionHostResult {
    SessionCode(SessionCode),
    SessionAnswerForward(SessionAnswerForward),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCode {
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionAnswerForward {
    pub answer: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCheck {
    pub result: SessionCheckResult,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SessionCheckResult {
    Success(SessionHost),
    WrongPassword,
    NotFound,
}