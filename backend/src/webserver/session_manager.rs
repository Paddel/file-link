use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use super::session::{Session, CondvarDetails};
use crate::shared::{HostCreate, HostCreateResult, ClientGetDetailsResult, ClientJoin, ClientJoinResult};

use rand::Rng;

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, session_create: HostCreate, address: SocketAddr) -> HostCreateResult {
        let code = self.generate_code();
        let session = Session::from(session_create, address);
        self.sessions.insert(code.clone(), session);
        HostCreateResult { code }
    }

    pub fn get_condvar_details(&self, address: &SocketAddr, code: &str) -> Option<Arc<CondvarDetails>> {
        let session = self.sessions.get(code)?;
        if session.address != *address {
            return None;
        }
        Some(session.condvar_details.clone())
    }

    pub fn get_connection_details(&self, code: &str, password: &str) -> Option<ClientGetDetailsResult> {
        let session = self.sessions.get(code)?;
        if session.password != password {
            return None;
        }
        
        let result = ClientGetDetailsResult { connection_details: session.connection_details_host.clone() };
        Some(result)
    }

    pub fn join_session(&self, session_join: ClientJoin) -> Option<ClientJoinResult> {
        let session = self.sessions.get(&session_join.code)?;
        if session.password != session_join.password {
            return None;
        }
        
        session.condvar_details.0.notify_all();

        Some(ClientJoinResult {
            compression_level: session.compression_level,
            has_password: session.has_password(),
            connection_details: session.connection_details_host.clone(),
        })
    }

    pub fn get_session(&self, code: &str) -> Option<&Session> {
        self.sessions.get(code)
    }

    fn generate_code(&self) -> String {
        const CODE_CHAR_SET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

        let code: String = (0..10)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..CODE_CHAR_SET.len());
                CODE_CHAR_SET.chars().nth(idx).unwrap()
            })
            .collect();
        code
    }
}
