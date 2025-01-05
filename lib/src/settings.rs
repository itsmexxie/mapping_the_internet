use config::Config;
#[cfg(feature = "serde")]
use serde::Deserialize;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct SettingsDatabase {
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct SettingsAPI {
    pub port: u16,
    #[cfg_attr(feature = "serde", serde(default = "default_api_auth"))]
    pub auth: bool,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct SettingsPokedex {
    pub address: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct SettingsUnit {
    pub username: String,
    pub password: String,
    pub address: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub announce_port: bool,
}

#[cfg(feature = "serde")]
fn default_api_auth() -> bool {
    true
}

#[cfg(feature = "serde")]
pub fn deserialize_from_config<S, RefStr: AsRef<str>>(filename: RefStr) -> (Config, S)
where
    S: for<'a> Deserialize<'a>,
{
    let config = match Config::builder()
        .add_source(config::File::with_name(filename.as_ref()))
        .build()
    {
        Ok(config) => config,
        Err(error) => {
            panic!(
                "{}",
                match error {
                    config::ConfigError::NotFound(_) =>
                        format!("{} was not found!", filename.as_ref()),
                    config::ConfigError::FileParse { uri: _, cause: _ } =>
                        format!("Failed to parse {}", filename.as_ref()),
                    _ => format!(
                        "Unknown error while trying to read {} ({})!",
                        filename.as_ref(),
                        error
                    ),
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
