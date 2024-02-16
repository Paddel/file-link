use std::sync::RwLock;

use rocket::{routes, Config};
use tokio::runtime::Runtime;
use unescape::unescape;
use super::routing::*;

use super::session_manager::SessionManager;

pub mod webserver {
    use super::*;

    pub fn run() {
        let config = Config {
            ..Config::default()
        };
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            rocket::custom(config)
                .manage(RwLock::new(SessionManager::new()))
                .mount("/", routes![root, create_session, poll_session, get_session_details, join_session, catch_all])
                .launch()
                .await
                .unwrap();
        });
    }

    pub fn unescape_quotes(s: &str) -> String {
        let s = s.trim_matches('"');
        unescape(s).unwrap()
    }
}