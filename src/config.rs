use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
}

pub fn get_config() -> Config {
    let data = fs::read_to_string("config.toml").expect("Unable to read config file");
    toml::from_str(&data).unwrap()
}
