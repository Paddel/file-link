use std::collections::HashMap;

use super::session::Session;
use crate::shared::{SessionCreate, SessionCreateResult, SessionJoin, SessionJoinResult};

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

    pub fn create_session(&mut self, session_create: SessionCreate) -> SessionCreateResult {
        let code = self.generate_code();
        let session = Session::from(session_create);
        self.sessions.insert(code.clone(), session);
        SessionCreateResult { code }
    }

    pub fn join_session(&self, session_join: SessionJoin) -> Option<SessionJoinResult> {
        let session = self.sessions.get(&session_join.code)?;
        if session.password != session_join.password {
            return None;
        }
        Some(SessionJoinResult {
            compression_level: session.compression_level,
            has_password: session.has_password(),
            connection_details: session.connection_details.clone(),
        })
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
