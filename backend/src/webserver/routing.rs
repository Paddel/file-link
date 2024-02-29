use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use rocket::fs::NamedFile;
use crate::shared::{HostPollResult, ClientGetDetails, HostCreate, ClientJoin, ClientJoinResult};
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

    let condvar_details = {
        let session_manager = session_manager.read();
        if session_manager.is_err() {
            return Err(Status::InternalServerError);
        }
        let session_manager = session_manager.unwrap();
        if !session_manager.is_session_owner(&address, &session_id) {
            return Err(Status::Forbidden);
        }

        let condvar_details = session_manager.get_condvar_details(&session_id);
        if condvar_details.is_none() {
            return Err(Status::NotFound);
        }
        condvar_details.unwrap()
    };

    let lock = condvar_details.1.lock().await;
    let _ = condvar_details.0.wait((lock, &condvar_details.1)).await;
    let lock = condvar_details.1.lock().await;
    let connection_details = lock.clone().expect("Details not set by client").clone();
    let result = HostPollResult { connection_details };
    let result = serde_json::to_string(&result).unwrap();
    Ok(result)
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

#[post("/api/sessions/details", data = "<data>")]
pub fn get_session_details(session_manager: &State<RwLock<SessionManager>>, data: String) -> Result<String, Status> {
    let data = unescape_quotes(&data);
    let session_join = serde_json::from_str::<ClientGetDetails>(&data);
    let session_join = match session_join {
        Ok(session_join) => session_join,
        Err(_) => return Err(Status::BadRequest),
    };

    let session_manager = session_manager.read();
    if session_manager.is_err() {
        return Err(Status::InternalServerError);
    }
    let session_manager = session_manager.unwrap();

    if !session_manager.is_session_code_valid(&session_join.code) {
        return Err(Status::NotFound);
    }

    let result = session_manager.get_connection_details(&session_join.code, &session_join.password);
    let result = match result {
        Some(result) => result,
        None => return Err(Status::Unauthorized),
    };
    let result = serde_json::to_string(&result).unwrap();
    Ok(result)
}

#[post("/api/sessions/join", data = "<data>")]
pub async fn join_session(
    session_manager: &State<RwLock<SessionManager>>,
    data: String,
) -> Result<String, Status> {
    let data = unescape_quotes(&data);
    let session_join = serde_json::from_str::<ClientJoin>(&data);
    let session_join = match session_join {
        Ok(session_join) => session_join,
        Err(_) => return Err(Status::BadRequest),
    };

    let (condvar_details, join_result) = {
        let session_manager = session_manager.read();
        if session_manager.is_err() {
            return Err(Status::InternalServerError);
        }
        let session_manager = session_manager.unwrap();

        let session = session_manager.get_session(&session_join.code);
        if session.is_none() {
            return Err(Status::NotFound);
        }
        let session = session.unwrap();

        if session.password != session_join.password {
            return Err(Status::Unauthorized);
        }

        let condvar_details = session.condvar_details.clone();
        let join_result = ClientJoinResult {
            compression_level: session.compression_level,
            has_password: session.has_password(),
            connection_details: session.connection_details_host.clone(),
        };
        (condvar_details, join_result)
    };
    
    *condvar_details.1.lock().await = Some(session_join.connection_details.clone());
    condvar_details.0.notify_all();
    let result = serde_json::to_string(&join_result).unwrap();
    Ok(result)
}
