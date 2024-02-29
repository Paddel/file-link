use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use super::session::{Session, CondvarDetails};
use crate::shared::{HostCreate, HostCreateResult, ClientGetDetailsResult};

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

    pub fn is_session_owner(&self, address: &SocketAddr, code: &str) -> bool {
        let session = self.sessions.get(code);
        if session.is_none() {
            return false;
        }
        let session = session.unwrap();
        session.address == *address
    }

    pub fn get_condvar_details(&self, code: &str) -> Option<Arc<CondvarDetails>> {
        let session = self.sessions.get(code)?;
        Some(session.condvar_details.clone())
    }

    pub fn is_session_code_valid(&self, code: &str) -> bool {
        self.sessions.contains_key(code)
    }

    pub fn get_connection_details(&self, code: &str, password: &str) -> Option<ClientGetDetailsResult> {
        let session = self.sessions.get(code)?;
        if session.password != password {
            return None;
        }
        
        let result = ClientGetDetailsResult { connection_details: session.connection_details_host.clone() };
        Some(result)
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
