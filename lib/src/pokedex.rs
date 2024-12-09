use reqwest::StatusCode;
use serde::Deserialize;

pub mod config;

use config::PokedexConfig;

#[derive(Deserialize)]
struct PokedexLoginResponse {
    token: String,
}

#[derive(Deserialize)]
pub struct Service {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ServiceUnit {
    pub id: String,
    pub service_id: i32,
    pub address: Option<String>,
    pub port: Option<i32>,
}

pub struct Pokedex {
    config: PokedexConfig,
    client: reqwest::Client,
    token: Option<String>,
}

impl Pokedex {
    pub fn new(config: PokedexConfig) -> Self {
        Pokedex {
            config,
            client: reqwest::Client::new(),
            token: None,
        }
    }

    pub async fn login(&mut self) -> Result<String, &'static str> {
        let mut pokedex_url = self.config.address.clone();
        pokedex_url.set_path("auth/login");

        let mut pokedex_query = pokedex_url.query_pairs_mut();
        pokedex_query.append_pair("username", &self.config.unit.username);
        pokedex_query.append_pair("password", &self.config.unit.password);

        if let Some(address) = &self.config.unit.address {
            pokedex_query.append_pair("address", &address.to_string());
        }

        if let Some(port) = &self.config.unit.port {
            pokedex_query.append_pair("port", &port.to_string());
        }

        drop(pokedex_query);

        match self.client.post(pokedex_url).send().await {
            Ok(res) => match res.status() {
                StatusCode::OK => {
                    let json = res.json::<PokedexLoginResponse>().await.unwrap();
                    self.token = Some(json.token.clone());
                    Ok(json.token)
                }
                StatusCode::BAD_REQUEST => Err("Invalid IP address and/or port in Pokedex login!"),
                StatusCode::UNAUTHORIZED => {
                    Err("Wrong unit username and/or password in Pokedex login!")
                }
                StatusCode::FORBIDDEN => Err("Service unit count limit exceeded!"),
                StatusCode::NOT_FOUND => Err("Incorrect Pokedex URL"),
                _ => Err("Unknown error while logging into Pokedex!"),
            },
            Err(_) => Err("Unknown error while logging into Pokedex!"),
        }
    }

    pub async fn logout(&self) {
        let mut pokedex_url = self.config.address.clone();
        pokedex_url.set_path("auth/logout");

        self.client
            .post(pokedex_url)
            .bearer_auth(
                self.token
                    .as_ref()
                    .expect("Can't logout if never logged in!"),
            )
            .send()
            .await
            .unwrap();
    }

    pub async fn get_services(&self) -> Result<Vec<Service>, StatusCode> {
        let mut pokedex_url = self.config.address.clone();
        pokedex_url.set_path("services");

        match self
            .client
            .get(pokedex_url)
            .bearer_auth(self.token.as_ref().expect(
                "Client must be logged in before attempting to send authenticated requests!",
            ))
            .send()
            .await
        {
            Ok(res) => {
                let status = res.status();
                if status == StatusCode::OK {
                    return Ok(res.json::<Vec<Service>>().await.unwrap());
                }
                return Err(status);
            }
            Err(e) => Err(e.status().unwrap()),
        }
    }

    pub async fn get_service_units(&self, service_id: i32) -> Result<Vec<ServiceUnit>, StatusCode> {
        let mut pokedex_url = self.config.address.clone();
        pokedex_url.set_path(&format!("services/{}/units", service_id.to_string()));

        match self
            .client
            .get(pokedex_url)
            .bearer_auth(self.token.as_ref().expect(
                "Client must be logged in before attempting to send authenticated requests!",
            ))
            .send()
            .await
        {
            Ok(res) => {
                let status = res.status();
                if status == StatusCode::OK {
                    return Ok(res.json::<Vec<ServiceUnit>>().await.unwrap());
                }
                return Err(status);
            }
            Err(e) => Err(e.status().unwrap()),
        }
    }
}
