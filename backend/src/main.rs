use once_cell::sync::Lazy;
use rocket::fs::{FileServer, NamedFile};
use rocket::routes;
use std::path::PathBuf;
use tokio::runtime::Runtime;

mod encryption;
mod networking;
mod util;
mod web_interface;

const BASE_PATH: &str = "src/web_interface/static/";
static INDEX_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(BASE_PATH).join("index.html"));

#[rocket::get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(&*INDEX_PATH).await.ok()
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        networking::initialize_networking();
        rocket::build()
            .mount("/", routes![index])
            .mount("/static", FileServer::from(BASE_PATH))
            .launch()
            .await
            .unwrap();
    });
}