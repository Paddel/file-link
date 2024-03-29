#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HostCreate {
    pub connection_details: String,
    pub compression_level: u8,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HostCreateResult {
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HostPollResult {
    pub connection_details: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientGetDetails {
    pub code: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientGetDetailsResult {
    pub connection_details: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientJoin {
    pub code: String,
    pub password: String,
    pub connection_details: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientJoinResult {
    pub compression_level: u8,
    pub has_password: bool,
    pub connection_details: String,
}