mod webserver;

use webserver::webserver::Webserver;

pub mod shared {
    use serde::{Deserialize, Serialize};
    use toml;
    include!("../../shared/api_protocol.rs");
    include!("../../shared/ws_protocol.rs");
    include!("../../shared/config.rs");
}

fn main() {
    let mut webserver = Webserver::new();
    webserver.run();
}