extern crate toml;

use std::fs::File;
use std::io::Read;
use std::env;
use std::path::Path;
use std::fs::write;

fn main() {
    let mut config_file = File::open("../config.toml").expect("Config file not found");
    let mut config_data = String::new();
    config_file.read_to_string(&mut config_data).expect("Unable to read config file");

    let config: toml::Value = toml::from_str(&config_data).expect("Error parsing config file");

    let server_address = config["server_address"].as_str().expect("Server address not found");

    let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join("config.rs");
    write(&output_path, format!("pub const SERVER_ADDRESS: &str = \"{}\";", server_address))
        .expect("Unable to write config file");
}