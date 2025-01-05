use mtilib::settings::SettingsDatabase;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: SettingsDatabase,
}
