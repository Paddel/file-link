use std::{net::SocketAddr, sync::Arc};

use async_condvar_fair::Condvar;
use tokio::sync::Mutex;

use crate::shared::HostCreate;

pub type CondvarDetails = (Condvar, Mutex<Option<String>>);

pub struct Session {
    pub compression_level: u8,
    pub password: String,
    pub connection_details_host: String,
    pub address: SocketAddr,
    pub condvar_details: Arc<CondvarDetails>,
}

impl Session {
    pub fn from(session_create: HostCreate, address: SocketAddr) -> Self {
        Self {
            compression_level: session_create.compression_level,
            password: session_create.password,
            connection_details_host: session_create.connection_details,
            address,
            condvar_details: Arc::new((Condvar::new(), Mutex::new(None))),
        }
    }

    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
}
