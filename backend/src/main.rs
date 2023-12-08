use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

use rocket::Config;
use rocket::fs::NamedFile;
use rocket::{get, routes};
use tokio::runtime::Runtime;

mod networking;

const INDEX_FILE_PATH: &str = "./public/index.html";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(INDEX_FILE_PATH));

pub mod shared {
    use serde::{Deserialize, Serialize};
    use toml;
    include!("../../shared/ws_protocol.rs");
    include!("../../shared/config.rs");
}

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