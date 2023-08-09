use rocket::{get, launch, routes};

mod encryption;
mod networking;
mod web_interface;
mod util;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the P2P File Transfer App!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(web_interface::upload_file::stage())
        .attach(web_interface::download_file::stage())
        .manage(initialize_networking())
}

fn initialize_networking() -> networking::NetworkManager {
    networking::NetworkManager::new()
}
