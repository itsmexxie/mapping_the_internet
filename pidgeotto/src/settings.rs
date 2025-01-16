use mtilib::settings::{SettingsAPI, SettingsDatabase, SettingsPokedex, SettingsUnit};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub database: SettingsDatabase,
    pub pokedex: SettingsPokedex,
    pub scanner: SettingsScanner,
    pub unit: SettingsUnit,
}

#[derive(Debug, Deserialize)]
pub struct SettingsScanner {
    #[serde(default = "_default_scanner_batch")]
    pub batch: u32,
    #[serde(default = "_default_scanner_max_tasks")]
    pub max_tasks: usize,
    #[serde(default = "_default_scanner_start")]
    pub start: String,
}

const fn _default_scanner_batch() -> u32 {
    1024
}

const fn _default_scanner_max_tasks() -> usize {
    512
}

fn _default_scanner_start() -> String {
    String::from("0.0.0.0")
}
