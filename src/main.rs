use rocket::{get};
use rocket::fs::NamedFile;
use std::path::PathBuf;
use tokio::runtime::Runtime;

mod encryption;
mod networking;
mod web_interface;
mod util;

const BASE_PATH_STR: &str = "src/web_interface/static/";

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(PathBuf::from(BASE_PATH_STR).join("index.html")).await.ok()
}

async fn initialize_networking() -> networking::NetworkManager {
    tokio::spawn(networking::start_signaling_server()).await;
    networking::NetworkManager::new()
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    // Your async code here
}