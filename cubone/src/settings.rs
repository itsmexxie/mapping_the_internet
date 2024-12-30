use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub api: SettingsAPI,
    pub database: SettingsDatabase,
    pub pokedex: SettingsPokedex,
    pub unit: SettingsUnit,
}

impl Settings {
    pub fn load() -> (Config, Settings) {
        let config = match Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .build()
        {
            Ok(config) => config,
            Err(error) => {
                panic!(
                    "{}",
                    match error {
                        config::ConfigError::NotFound(_) => "config.toml not found!",
                        config::ConfigError::FileParse { uri: _, cause: _ } =>
                            "Failed to parse config.toml",
                        _ => "Unknown error while trying to read config.toml!",
                    }
                );
            }
        };

        (
            config.clone(),
            config
                .try_deserialize()
                .expect("Failed to parse configuration!"),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct SettingsAPI {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SettingsDatabase {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct SettingsPokedex {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct SettingsUnit {
    pub username: String,
    pub password: String,
    pub address: Option<String>,
    pub port: Option<u16>,
}
