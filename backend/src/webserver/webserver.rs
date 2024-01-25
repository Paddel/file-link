use rocket::{routes, Config};
use tokio::runtime::Runtime;
use unescape::unescape;
use super::routing::*;


pub struct Webserver {
    
}

impl Webserver {
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn run(&mut self) {
        let config = Config {
            ..Config::default()
        };
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            rocket::custom(config)
                .mount("/", routes![root, create_session, catch_all])
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