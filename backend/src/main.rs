use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

use rocket::Config;
use rocket::fs::NamedFile;
// use rocket::response::status::NotFound;
use rocket::http::Status;
use rocket::{get, post, routes};
use tokio::runtime::Runtime;
use crate::shared::SessionCreate;
use unescape::unescape;

mod networking;

const INDEX_FILE_PATH: &str = "./public/index.html";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(INDEX_FILE_PATH));

pub mod shared {
    use serde::{Deserialize, Serialize};
    use toml;
    include!("../../shared/api_protocol.rs");
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

#[post("/api/sessions", data = "<data>")]
async fn create_session(data: String) -> Result<String, Status> {
    let data = unescape_quotes(&data);
    let session_create = serde_json::from_str::<SessionCreate>(&data);
    let session_create = match session_create {
        Ok(session_create) => session_create,
        Err(_) => return Err(Status::BadRequest),
    };
    
    println!("Session create: {:?}", session_create);
    Ok(data)
}

fn unescape_quotes(s: &str) -> String {
    let s = s.trim_matches('"');
    unescape(s).unwrap()
}

fn main() {
    let config = Config {
        ..Config::default()
    };
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        networking::NetworkManager::initialize_networking();
        rocket::custom(config)
            .mount("/", routes![root, create_session, catch_all])
            .launch()
            .await
            .unwrap();
    });
}