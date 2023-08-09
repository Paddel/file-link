pub mod upload_file {
    use rocket::fairing::AdHoc;

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("Upload File", |rocket| async {
            rocket
        })
    }
}

pub mod download_file {
    use rocket::fairing::AdHoc;

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("Download File", |rocket| async {
            rocket
        })
    }
}