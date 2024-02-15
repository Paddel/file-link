 use std::{net::SocketAddr, sync::{Condvar, Mutex}};

use crate::shared::HostCreate;

pub struct Session {
    pub compression_level: u8,
    pub password: String,
    pub connection_details_host: String,
    pub address: SocketAddr,
    pub connection_details_join: Mutex<Option<String>>,
    pub join_cond: Condvar,
}

impl Session {
    pub fn from(session_create: HostCreate, address: SocketAddr) -> Self {
        Self {
            compression_level: session_create.compression_level,
            password: session_create.password,
            connection_details_host: session_create.connection_details,
            address,
            connection_details_join: Mutex::new(None),
            join_cond: Condvar::new(),
        }
    }

    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
}