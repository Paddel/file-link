#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionCreate {
    pub connection_details: String,
    pub compression_level: u8,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionCreateResult {
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionJoin {
    pub code: &str,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionJoinResult {
    pub compression_level: u8,
    pub has_password: bool,
    pub connection_details: String,
}
