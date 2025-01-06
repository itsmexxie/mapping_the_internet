use mtilib::settings::{SettingsAPI, SettingsPokedex, SettingsUnit};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub pokedex: SettingsPokedex,
    pub unit: SettingsUnit,
}
