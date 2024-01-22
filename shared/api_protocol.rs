#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionCreate {
    pub compression_level: u8,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionCreateResult {
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionJoin {
    pub code: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionJoinResult {
    pub compression_level: u8,
    pub offer: String,
    pub has_password: bool,
}
