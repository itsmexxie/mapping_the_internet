use mtilib::settings::{SettingsAPI, SettingsDatabase, SettingsPokedex, SettingsUnit};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub database: SettingsDatabase,
    pub pokedex: SettingsPokedex,
    pub unit: SettingsUnit,
}
