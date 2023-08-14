pub mod upload_file {
    use rocket::fairing::AdHoc;
    // ... other imports ...

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("Upload File", |rocket| async {
            // Route configuration here
            rocket
        })
    }
}

pub mod download_file {
    use rocket::fairing::AdHoc;
    // ... other imports ...

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("Download File", |rocket| async {
            // Route configuration here
            rocket
        })
    }
}