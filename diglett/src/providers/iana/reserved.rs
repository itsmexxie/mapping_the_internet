use std::{path::Path, str::FromStr};

use config::Config;
use tracing::info;

use crate::{providers, utils::CIDR};

pub struct ReservedProvider {
    pub value: Vec<CIDR>,
}

impl ReservedProvider {
    pub async fn load(config: &Config) -> ReservedProvider {
        info!("Loading IANA reserved addresses...");

        // Check if we need to redownload file
        match providers::load_provider_sources(config, "iana.reserved") {
            Some(sources) => {
                crate::providers::check_and_download(&sources).await;

                let mut reserved_blocks = Vec::new();
                for source in sources {
                    let source_filepath = Path::new(&source.filepath);

                    let mut reader =
                        csv::Reader::from_reader(std::fs::File::open(source_filepath).unwrap());

                    for reader_line in reader.deserialize() {
                        let addresses: String = reader_line.unwrap();

                        for address in addresses.split(",").map(|x| x.trim()) {
                            let address = address.split(" ").collect::<Vec<_>>()[0];
                            reserved_blocks.push(CIDR::from_str(address).unwrap());
                        }
                    }
                }

                // Manually add multicast block
                // TODO: Move into its own section (create a new allocation state)
                reserved_blocks.push(CIDR::from_str("224.0.0.0/4").unwrap());

                info!("Loaded IANA reserved addresses!");
                reserved_blocks.sort();

                ReservedProvider {
                    value: reserved_blocks,
                }
            }
            None => panic!("Failed to load sources for IANA reserved addresses!"),
        }
    }
}
