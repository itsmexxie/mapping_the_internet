use mtilib::settings::{SettingsAPI, SettingsDatabase};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub database: SettingsDatabase,
    pub unit: SettingsUnit,
}

#[derive(Debug, Deserialize)]
pub struct SettingsUnit {
    #[serde(default = "_default_unit_username")]
    pub username: String,
    pub address: String,
    #[serde(default)]
    pub announce_port: bool,
}

fn _default_unit_username() -> String {
    String::from("pokedex")
}
