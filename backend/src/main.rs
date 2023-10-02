use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::fs;

use rocket::fs::NamedFile;
use rocket::{get, routes};
use tokio::runtime::Runtime;

use rocket::Config;

mod networking;

pub mod shared {
    use serde::{Deserialize, Serialize};
    include!("../../shared/ws_protocol.rs");
}

const BASE_PATH: &str = "./public/";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(BASE_PATH).join("index.html"));

#[get("/")]
async fn root() -> Option<NamedFile> {
    NamedFile::open(&*INDEX_PATH).await.ok()
}

#[get("/<path..>")]
async fn catch_all(path: PathBuf) -> Option<NamedFile> {
    let path = match fs::metadata(path.clone()) {
        Ok(_) => &path,
        Err(_) => &*INDEX_PATH,
    };

    NamedFile::open(&path).await.ok()
}

fn main() {
    let config = Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 80,
        ..Config::default()
    };
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        networking::NetworkManager::initialize_networking();
        rocket::custom(config)
            .mount("/", routes![root, catch_all])
            .launch()
            .await
            .unwrap();
    });
}