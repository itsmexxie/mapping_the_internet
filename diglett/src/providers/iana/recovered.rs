use std::{fmt::Display, net::Ipv4Addr, path::Path, str::FromStr};

use config::Config;
use mtilib::types::Rir;
use tracing::info;

use crate::providers;

pub struct RecoveredProvider {
    pub value: Vec<RecoveredEntry>,
}

impl RecoveredProvider {
    pub async fn load(config: &Config) -> RecoveredProvider {
        info!("Loading IANA recovered addresses...");

        // Check if we need to redownload file
        match providers::load_provider_sources(config, "iana.recovered") {
            Some(sources) => {
                providers::check_and_download(&sources).await;

                let mut recovered_entries = Vec::new();
                for source in sources {
                    // Get the configured filepath and read the file into memory
                    let source_filepath = Path::new(&source.filepath);

                    let mut reader =
                        csv::Reader::from_reader(std::fs::File::open(source_filepath).unwrap());

                    // Parse
                    for reader_line in reader.deserialize() {
                        let entry: (String, String, String) = reader_line.unwrap();

                        recovered_entries.push(RecoveredEntry {
                            start: Ipv4Addr::from_str(&entry.0).unwrap(),
                            end: Ipv4Addr::from_str(&entry.1).unwrap(),
                            rir: Rir::from_str(&entry.2).unwrap(),
                        });
                    }
                }

                info!("Loaded IANA recovered addresses!");

                RecoveredProvider {
                    value: recovered_entries,
                }
            }
            None => panic!("Failed to load sources for IANA recovered addresses!"),
        }
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
