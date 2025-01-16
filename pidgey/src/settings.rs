use mtilib::settings::{SettingsAPI, SettingsPokedex, SettingsUnit};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub diglett: Option<SettingsDiglett>,
    #[serde(default = "_default_max_workers")]
    pub max_workers: usize,
    pub pidgeotto: Option<SettingsPidgeotto>,
    pub pokedex: SettingsPokedex,
    pub unit: SettingsUnit,
}

const fn _default_max_workers() -> usize {
    64
}

#[derive(Debug, Deserialize)]
pub struct SettingsDiglett {
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsPidgeotto {
    pub address: Option<String>,
    #[serde(default = "_default_pidgeotto_connect")]
    pub connect: bool,
}

const fn _default_pidgeotto_connect() -> bool {
    true
}
