use std::{fmt::Display, path::Path, str::FromStr};

use config::Config;
use mtilib::types::Rir;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{providers, utils::CIDR};

pub struct RirAllocationsProvider {
    pub value: Vec<RirAllocationEntry>,
}

impl RirAllocationsProvider {
    pub async fn load(config: &Config) -> Self {
        info!("Loading RIR allocations...");

        // Check if we need to redownload file
        match providers::load_provider_sources(config, "thyme.rir_allocations") {
            Some(sources) => {
                providers::check_and_download(&sources).await;

                let mut rir_allocations = Vec::new();
                for source in sources {
                    let rir_allocations_filepath = Path::new(&source.filepath);

                    let mut file: File = File::open(rir_allocations_filepath).await.unwrap();
                    let mut contents_str = String::new();
                    file.read_to_string(&mut contents_str).await.unwrap();

                    let re = Regex::new(r"[\t ]+(\d+\/\d)[\t ]+(.+)").unwrap();
                    for (_, [prefix, rir]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
                        let parsed_cidr = CIDR::from_str(prefix).unwrap();
                        rir_allocations.push(RirAllocationEntry {
                            cidr: parsed_cidr,
                            rir: Rir::from_str(rir).unwrap(),
                        });
                    }
                }

                info!("Loaded RIR allocations!");
                rir_allocations.sort();

                RirAllocationsProvider {
                    value: rir_allocations,
                }
            }
            None => panic!("Failed to load sources for Thyme's RIR allocations!"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RirAllocationEntry {
    pub cidr: CIDR,
    pub rir: Rir,
}

impl PartialOrd for RirAllocationEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RirAllocationEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cidr.cmp(&self.cidr)
    }
}

impl Display for RirAllocationEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "address: {}, mask: {}, rir: {}",
            self.cidr.prefix,
            self.cidr.mask,
            self.rir.to_string()
        ))
    }
}
