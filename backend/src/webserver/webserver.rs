use std::sync::RwLock;

use super::routing::*;
use super::session_manager::SessionManager;
use rocket::{routes, Config};
use tokio::runtime::Runtime;
use unescape::unescape;

use crate::shared::BACKEND_CONFIG;
use std::net::IpAddr;

pub mod webserver {
    use super::*;

    pub fn run() {
        let backend_config = &*BACKEND_CONFIG;
        let web_bind_addr = backend_config.web_bind_addr.clone();
        let web_bind_addr: IpAddr = web_bind_addr.parse().expect("Invalid IP address");
        let web_port = backend_config.web_port;
        let config = Config {
            address: web_bind_addr,
            port: web_port,
            ..Config::default()
        };
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            rocket::custom(config)
                .manage(RwLock::new(SessionManager::new()))
                .mount(
                    "/",
                    routes![
                        root,
                        create_session,
                        poll_session,
                        get_session_details,
                        join_session,
                        catch_all
                    ],
                )
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
