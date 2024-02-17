mod webserver;

pub mod shared {
    use serde::{Deserialize, Serialize};
    use toml;
    include!("../../shared/api_protocol.rs");
    include!("../../shared/config.rs");
}

fn main() {
    webserver::webserver::webserver::run();
}