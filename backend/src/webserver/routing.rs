use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use rocket::fs::NamedFile;
use crate::shared::HostCreate;
use rocket::http::Status;
use rocket::{get, post, State};

use super::session_manager::SessionManager;
use super::webserver::webserver::unescape_quotes;
use std::net::SocketAddr;

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
pub async fn poll_session(
    address: SocketAddr,
    session_manager: &State<RwLock<SessionManager>>,
    session_id: String,
) -> Result<String, Status> {
    print!("Polling session: {}", session_id);
    let session_manager = session_manager.write();
    if session_manager.is_err() {
        return Err(Status::InternalServerError);
    }
    let session_manager = session_manager.unwrap();

    let session = session_manager.get_session(&session_id);
    if session.is_none() {
        return Err(Status::NotFound);
    }
    let session = session.unwrap();

    if session.address != address {
        return Err(Status::Forbidden);
    }

    let connection_details = &*session
        .join_cond
        .wait_while(session.connection_details_join.lock().unwrap(), |details| {
            details.is_none()
        }).unwrap();

    if connection_details.is_none() {
        return Err(Status::NotFound);
    }

    print!("Connection details: {}", connection_details.clone().unwrap());
    
    Ok(connection_details.clone().unwrap())
}

#[post("/api/sessions", data = "<data>")]
pub async fn create_session(
    address: SocketAddr,
    session_manager: &State<RwLock<SessionManager>>,
    data: String,
) -> Result<String, Status> {
    let data = unescape_quotes(&data);
    let session_create = serde_json::from_str::<HostCreate>(&data);
    let session_create = match session_create {
        Ok(session_create) => session_create,
        Err(_) => return Err(Status::BadRequest),
    };

    let session_manager = session_manager.write();
    if session_manager.is_err() {
        return Err(Status::InternalServerError);
    }
    let mut session_manager = session_manager.unwrap();

    let result = session_manager.create_session(session_create, address);
    let result = serde_json::to_string(&result).unwrap();
    Ok(result)
}
