use std::sync::RwLock;

use super::routing::*;
use super::session_manager::SessionManager;
use rocket::routes;
use rocket::config::{Config, TlsConfig, MutualTls};
use tokio::runtime::Runtime;
use unescape::unescape;

use crate::shared::BACKEND_CONFIG;
use std::net::IpAddr;

pub mod webserver {
    use super::*;

    pub fn run() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            rocket::custom(create_config())
                .manage(RwLock::new(SessionManager::new()))
                .mount(
                    "/",
                    create_routes(),
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

    fn create_routes() -> Vec<rocket::Route>{
        let backend_config = &*BACKEND_CONFIG;
        let mut routes: Vec<rocket::Route> = Vec::new();

        if backend_config.web_serve_page == "true" {
            let routes_page: Vec<rocket::Route> = routes![
                root,
                catch_all
            ];
            routes.extend(routes_page);
        }
        if backend_config.web_serve_api == "true" {
            let routes_api: Vec<rocket::Route> = routes![
                create_session,
                poll_session,
                get_session_details,
                join_session
            ];
            routes.extend(routes_api);
        }
        routes
    }

    fn create_config() -> Config {
        let backend_config = &*BACKEND_CONFIG;
        let web_bind_addr = backend_config.web_bind_addr.clone();
        let web_bind_addr: IpAddr = web_bind_addr.parse().expect("Invalid IP address");
        let web_port = backend_config.web_port;
        let web_tls_enabled = backend_config.web_tls_enabled == "true";

        let config_tls = {
            if web_tls_enabled {
                let tls_config = TlsConfig::from_paths(
                    backend_config.web_tls_certs.clone(),
                    backend_config.web_tls_key.clone()
                ).with_mutual(MutualTls::from_path(backend_config.web_tls_ca_cert.clone()));
                Some(tls_config)
            } else {
                None
            }
        };

        let config = Config {
            address: web_bind_addr,
            port: web_port,
            tls: config_tls,

            ..Config::default()
        };

        config
    }
}
