use once_cell::sync::Lazy;
use std::path::PathBuf;

use rocket::fs::{FileServer, NamedFile};
use rocket::{catch, catchers};
use tokio::runtime::Runtime;

mod encryption;
mod networking;
mod util;
mod web_interface;

const BASE_PATH: &str = "./public/static/";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(BASE_PATH).join("index.html"));

#[catch(default)]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(&*INDEX_PATH).await.ok()
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        networking::initialize_networking();
        rocket::build()
            .mount("/public", FileServer::from("./public"))
            .register("/", catchers![index])
            .launch()
            .await
            .unwrap();
    });
}