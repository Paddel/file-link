struct ApiService {

}

impl ApiService {
    fn new() -> ApiService {
        ApiService {}
    }

    pub fn post_session() {
        let  path = get_host_address() + "/api/sessions";
        let body = reqwest::get(path)
        .await?
        .text()
        .await?;
    }

    fn get_host_address() {
        let frontend_config = &*FRONTEND_CONFIG;
        frontend_config.api_ddress
    }
}