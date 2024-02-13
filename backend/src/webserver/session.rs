 use crate::shared::HostCreate;

pub struct Session {
    pub compression_level: u8,
    pub password: String,
    pub connection_details: String,
}

impl Session {
    pub fn from(session_create: HostCreate) -> Self {
        Self {
            compression_level: session_create.compression_level,
            password: session_create.password,
            connection_details: session_create.connection_details,
        }
    }

    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
}