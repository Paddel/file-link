/*
    rtcs_protocol.rs

    This module defines the Web-RTC protocol data structures used for communication between the host and client.
*/

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/* Host -> Client */
#[derive(Clone, Serialize, Deserialize)]
pub struct FilesUpdate {
    pub files: Vec<FileInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub uuid: Uuid,
    pub size: f64,
}

/* Client -> Host */
#[derive(Clone, Serialize, Deserialize)]
pub struct FileRequest {
    pub uuid: Uuid,
}