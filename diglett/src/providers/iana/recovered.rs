use std::{fmt::Display, net::Ipv4Addr, path::Path, str::FromStr};

use config::Config;
use mtilib::types::Rir;
use tracing::info;

use crate::get_config_value;

const SECTIONS: [&str; 1] = ["iana.recovered"];

pub struct RecoveredProvider {
    pub value: Vec<RecoveredEntry>,
}

impl RecoveredProvider {
    pub async fn load(config: &Config) -> RecoveredProvider {
        info!("Loading IANA recovered addresses...");

        // Check if we need to redownload file
        crate::providers::check_many_and_download(config, &SECTIONS).await;

        // Get the configured filepath and read the file into memory
        let section_filepath_cnf =
            &get_config_value::<String>(config, "providers.iana.recovered.filepath");
        let section_filepath = Path::new(section_filepath_cnf);

        let mut reader = csv::Reader::from_reader(std::fs::File::open(section_filepath).unwrap());

        // Parse
        let mut recovered_entries = Vec::new();

        for reader_line in reader.deserialize() {
            let entry: (String, String, String) = reader_line.unwrap();

            recovered_entries.push(RecoveredEntry {
                start: Ipv4Addr::from_str(&entry.0).unwrap(),
                end: Ipv4Addr::from_str(&entry.1).unwrap(),
                rir: Rir::from_str(&entry.2).unwrap(),
            });
        }

        info!("Loaded IANA recovered addresses!");

        RecoveredProvider {
            value: recovered_entries,
        }
    }

    pub async fn register_and_load(
        config: &Config,
        registered: &mut Vec<Vec<String>>,
    ) -> RecoveredProvider {
        registered.push(
            SECTIONS
                .clone()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
        );

        RecoveredProvider::load(config).await
    }
}

pub struct RecoveredEntry {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
    pub rir: Rir,
}

impl Display for RecoveredEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "start: {}, end: {}, rir: {}",
            self.start, self.end, self.rir
        ))
    }
}
