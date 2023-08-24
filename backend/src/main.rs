use once_cell::sync::Lazy;
use std::path::PathBuf;

use rocket::fs::{FileServer, NamedFile};
use rocket::{catch, catchers};
use tokio::runtime::Runtime;

mod web_interface;
use rocket::Config;

mod networking;

pub mod shared {
    use serde::{Deserialize, Serialize};
    include!("../../shared/ws_protocol.rs");
}

const BASE_PATH: &str = "./public/static/";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(BASE_PATH).join("index.html"));

#[catch(default)]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(&*INDEX_PATH).await.ok()
}

fn main() {
    let config = Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 8000,
        ..Config::default()
    };

    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        networking::NetworkManager::initialize_networking();
        rocket::custom(config)
            .mount("/public", FileServer::from("./public"))
            .register("/", catchers![index])
            .launch()
            .await
            .unwrap();
    });
}