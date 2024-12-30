use url::Url;

#[derive(Clone)]
pub struct PokedexConfig {
    pub unit: PokedexUnitConfig,
    pub address: Url,
    pub port: Option<u16>,
}

#[derive(Clone)]
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
            address: Url::parse(address).expect("Failed to parse pokedex address!"),
            port: None,
        }
    }

    pub fn with_port(unit: PokedexUnitConfig, address: &String, port: u16) -> Self {
        let mut config = PokedexConfig::new(unit, address);
        config.port = Some(port);
        config
    }

    #[cfg(feature = "config")]
    pub fn from_config(config: &config::Config) -> Self {
        let unit_username = config
            .get_string("unit.username")
            .expect("unit.username must be set!");
        let unit_password = config
            .get_string("unit.password")
            .expect("unit.password must be set!");
        // let unit_address = match config.get_string("unit.address") {
        //     Ok(address) => Some(Url::parse(&address).expect("Failed to parse unit address!")),
        //     Err(_) => None,
        // };
        let unit_address = config.get_string("unit.address").ok();
        let unit_port = match config.get_bool("unit.announce_port").unwrap_or(false) {
            true => Some(
                config
                    .get_int("api.port")
                    .expect("api.port must be set when unit.announce_port is set to true!")
                    as u16,
            ),
            false => None,
        };

        let pokedex_address = Url::parse(
            &config
                .get_string("pokedex.address")
                .expect("pokedex.address must be set!"),
        )
        .expect("Failed to parse pokedex address!");
        let pokedex_port = match config.get_int("pokedex.port") {
            Ok(port) => Some(port as u16),
            Err(_) => None,
        };

        PokedexConfig {
            unit: PokedexUnitConfig {
                username: unit_username,
                password: unit_password,
                address: unit_address,
                port: unit_port,
            },
            address: pokedex_address,
            port: pokedex_port,
        }
    }
}

impl PokedexUnitConfig {
    pub fn new(username: String, password: String) -> Self {
        PokedexUnitConfig {
            username,
            password,
            address: None,
            port: None,
        }
    }

    pub fn with_address(username: String, password: String, address: String) -> Self {
        let mut config = PokedexUnitConfig::new(username, password);
        config.address = Some(address);
        config
    }

    pub fn with_address_and_port(
        username: String,
        password: String,
        address: String,
        port: u16,
    ) -> Self {
        let mut config = PokedexUnitConfig::with_address(username, password, address);
        config.port = Some(port);
        config
    }
}
