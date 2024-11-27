use reqwest::Url;
use serde::Deserialize;

pub struct PokedexConfig {
    pub unit: PokedexUnitConfig,
    pub address: String,
    pub port: Option<u16>,
}

pub struct PokedexUnitConfig {
    pub username: String,
    pub password: String,
    pub address: Option<String>,
    pub port: Option<u16>,
}

impl PokedexConfig {
    pub fn new(unit: PokedexUnitConfig, address: &String) -> Self {
        PokedexConfig {
            unit,
            address: address.to_string(),
            port: None,
        }
    }

    pub fn with_port(unit: PokedexUnitConfig, address: &String, port: u16) -> Self {
        let mut config = PokedexConfig::new(unit, address);
        config.port = Some(port);
        config
    }
}

impl PokedexUnitConfig {
    pub fn new(username: &String, password: &String) -> Self {
        PokedexUnitConfig {
            username: username.to_string(),
            password: password.to_string(),
            address: None,
            port: None,
        }
    }

    pub fn with_address(username: &String, password: &String, address: &String) -> Self {
        let mut config = PokedexUnitConfig::new(username, password);
        config.address = Some(address.to_string());
        config
    }

    pub fn with_address_and_port(
        username: &String,
        password: &String,
        address: &String,
        port: u16,
    ) -> Self {
        let mut config = PokedexUnitConfig::with_address(username, password, address);
        config.port = Some(port);
        config
    }
}

pub fn pokedex_url(address: &String, port: Option<u16>) -> String {
    let mut pokedex_url = String::from(address);
    if let Some(port) = port {
        pokedex_url += ":";
        pokedex_url += &port.to_string();
    }

    return pokedex_url;
}

#[derive(Deserialize)]
struct PokedexLoginResponse {
    token: String,
}

pub async fn login(config: &PokedexConfig) -> Result<String, &'static str> {
    let pokedex_client = reqwest::Client::new();

    let mut pokedex_url = pokedex_url(&config.address, config.port);
    pokedex_url += "/auth/login";

    match pokedex_client
        .post(
            Url::parse_with_params(
                &pokedex_url,
                &[
                    ("username", config.unit.username.to_owned()),
                    ("password", config.unit.password.to_owned()),
                ],
            )
            .unwrap(),
        )
        .send()
        .await
    {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => {
                let json = res.json::<PokedexLoginResponse>().await.unwrap();
                Ok(json.token)
            }
            reqwest::StatusCode::BAD_REQUEST => {
                Err("Invalid IP address and/or port in Pokedex login!")
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                Err("Wrong unit username and/or password in Pokedex login!")
            }
            reqwest::StatusCode::FORBIDDEN => Err("Service unit count limit exceeded!"),
            _ => Err("Unknown error while logging into Pokedex!"),
        },
        Err(_) => Err("Unknown error while logging into Pokedex!"),
    }
}

pub async fn logout(config: &PokedexConfig, token: &String) {
    let pokedex_client = reqwest::Client::new();

    let mut pokedex_url = pokedex_url(&config.address, config.port);
    pokedex_url += "/auth/logout";

    pokedex_client
        .post(pokedex_url)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
}
