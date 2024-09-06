use std::fs::File;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    wifi_ssid: String,
    wifi_psk: String,
    server_address: String,
}

fn main() {
    let config_path = Path::new("config.json");

    if !config_path.exists() {
        panic!("config.json file not found");
    }

    let file = File::open(config_path).expect("Failed to open config.json!");
    let config: Config = serde_json::from_reader(file).expect("Failed to parse config.json!");

    println!("cargo:rustc-env=WIFI_SSID={}", config.wifi_ssid);
    println!("cargo:rustc-env=WIFI_PSK={}", config.wifi_psk);
    println!("cargo:rustc-env=SERVER_ADDRESS={}", config.server_address);
    println!("cargo:rerun-if-changed=config.json");
}
