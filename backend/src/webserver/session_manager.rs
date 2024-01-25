use std::collections::HashMap;

use rand::Rng;


struct Session {
    
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self) -> String {
        let code = self.generate_code();
        self.sessions.insert(code.clone(), Session {});
        self.sessions.insert(code.clone(), Session {});
        code
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