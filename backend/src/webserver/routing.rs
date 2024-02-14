use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use rocket::fs::NamedFile;
// use rocket::response::status::NotFound;
use rocket::http::Status;
use crate::shared::HostCreate;
use rocket::{get, post, State};

use super::session_manager::SessionManager;
use super::webserver::webserver::unescape_quotes;

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

#[get("/api/sessions/poll/<session_id>")]
pub async fn poll_session(session_manager: &State<RwLock<SessionManager>>, session_id: String) -> Result<String, Status> {
    print!("Polling session: {}", session_id);
    // let data = unescape_quotes(&data);
    // let session_create = serde_json::from_str::<HostCreate>(&data);
    // let session_create = match session_create {
    //     Ok(session_create) => session_create,
    //     Err(_) => return Err(Status::BadRequest),
    // };

    // let result = session_manager.write().unwrap().create_session(session_create);
    // let result = serde_json::to_string(&result).unwrap();
    // Ok(result)
    Err(Status::BadRequest)
}

#[post("/api/sessions", data = "<data>")]
pub async fn create_session(session_manager: &State<RwLock<SessionManager>>, data: String) -> Result<String, Status> {
    let data = unescape_quotes(&data);
    let session_create = serde_json::from_str::<HostCreate>(&data);
    let session_create = match session_create {
        Ok(session_create) => session_create,
        Err(_) => return Err(Status::BadRequest),
    };

    let result = session_manager.write().unwrap().create_session(session_create);
    let result = serde_json::to_string(&result).unwrap();
    Ok(result)
}