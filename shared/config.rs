use once_cell::sync::Lazy;
use std::{fs, process};

const CONFIG_FILE_PATH: &str = "config.toml";

pub static BACKEND_CONFIG: Lazy<BackendConfig> = Lazy::new(|| {
    let config: Config = strict_load_config(CONFIG_FILE_PATH);
    config.backend_config
});

pub static FRONTEND_CONFIG: Lazy<FrontendConfig> = Lazy::new(|| {
    let config: Config = strict_load_config(CONFIG_FILE_PATH);
    config.frontend_config
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend_config: BackendConfig,
    pub frontend_config: FrontendConfig,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub ws_bind_addr: String,
    pub ws_port: i32,
    // pub web_bind_addr: String,
    // pub web_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct FrontendConfig {
    pub api_address: String,
}

fn load_config<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: T = toml::from_str(&content)?;
    Ok(config)
}

fn strict_load_config<T: serde::de::DeserializeOwned>(path: &str) -> T {
    load_config::<T>(path).unwrap_or_else(|error| {
        let msg = format!("Failed to load config: {}", error);
        eprintln!("{}", msg);
        process::exit(1);
    })
}