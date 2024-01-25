use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

use rocket::fs::NamedFile;
// use rocket::response::status::NotFound;
use rocket::http::Status;
use crate::shared::SessionCreate;
use rocket::{get, post};

use super::webserver::Webserver;

const INDEX_FILE_PATH: &str = "./public/index.html";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(INDEX_FILE_PATH));

#[get("/")]
pub async fn root() -> Option<NamedFile> {
    NamedFile::open(&*INDEX_PATH).await.ok()
}

#[get("/<path..>")]
pub async fn catch_all(path: PathBuf) -> Option<NamedFile> {
    let path = match fs::metadata(path.clone()) {
        Ok(_) => &path,
        Err(_) => &*INDEX_PATH,
    };

    NamedFile::open(&path).await.ok()
}

#[post("/api/sessions", data = "<data>")]
pub async fn create_session(data: String) -> Result<String, Status> {
    let data = Webserver::unescape_quotes(&data);
    let session_create = serde_json::from_str::<SessionCreate>(&data);
    let session_create = match session_create {
        Ok(session_create) => session_create,
        Err(_) => return Err(Status::BadRequest),
    };
    
    println!("Session create: {:?}", session_create);
    Ok(data)
}