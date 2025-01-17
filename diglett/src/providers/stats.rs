use std::{fmt::Display, net::Ipv4Addr, path::Path, str::FromStr};

use config::Config;
use mtilib::types::{AllocationState, Rir};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{
    providers::{self, CheckAndDownloadSource},
    utils::CIDR,
};

use super::ProviderSource;

const HEADER_SIZE: u32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatsEntry {
    pub cidr: CIDR,
    pub allocation_state: AllocationState,
    pub rir: Rir,
    pub country: Option<String>,
}

impl PartialOrd for StatsEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StatsEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cidr.cmp(&self.cidr)
    }
}

impl Display for StatsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "cidr: {}, allocation_state: {}, rir: {}, country: {}",
            self.cidr,
            self.allocation_state,
            self.rir,
            self.country.as_ref().unwrap_or(&String::from("-"))
        ))
    }
}

pub struct StatsProvider {
    pub values: Vec<StatsEntry>,
    pub sources: Vec<ProviderSource>,
}

impl StatsProvider {
    async fn load_source(source: &ProviderSource) -> Vec<StatsEntry> {
        let source_filepath = Path::new(&source.filepath);

        let mut file = File::open(source_filepath).await.unwrap();
        let mut contents_str = String::new();
        file.read_to_string(&mut contents_str).await.unwrap();
        let lines = contents_str.split("\n").collect::<Vec<_>>();

        let mut stat_entries = Vec::new();

        let mut i: usize = 0;
        let mut parsed_header = false;
        let mut parsed_header_sections: u32 = 0;
        let mut ipv4_offset: u32 = 0;
        let mut ipv4_count: u32 = 0;
        let mut parsed_ipv4: u32 = 0;
        loop {
            if i > lines.len() {
                return stat_entries;
            }

            if lines[i].starts_with("#") {
                i += 1;
                continue;
            }

            if parsed_header {
                if parsed_ipv4 >= ipv4_count {
                    return stat_entries;
                }

                let parts = lines[i].split("|").collect::<Vec<_>>();
                let alloc_state = AllocationState::from_str(parts[6]).unwrap();
                let country = match alloc_state {
                    AllocationState::Allocated => Some(parts[1].to_string()),
                    _ => None,
                };

                stat_entries.push(StatsEntry {
                    cidr: CIDR {
                        prefix: Ipv4Addr::from_str(parts[3]).unwrap().into(),
                        mask: (u32::MAX & !(parts[4].parse::<u32>().unwrap() - 1)).count_ones()
                            as u16,
                    },
                    allocation_state: alloc_state,
                    rir: Rir::from_str(parts[0]).unwrap(),
                    country,
                });

                parsed_ipv4 += 1;
                i += 1;
            } else {
                let parts = lines[i].split("|").collect::<Vec<_>>();

                match parts[2] {
                    "ipv4" => {
                        parsed_header_sections += 1;
                        parsed_header = true;
                        ipv4_count = parts[4].parse::<u32>().unwrap();
                        i += (ipv4_offset + (HEADER_SIZE - parsed_header_sections) + 1) as usize;
                        // header size is always 4
                    }
                    "ipv6" | "asn" => {
                        parsed_header_sections += 1;
                        ipv4_offset += parts[4].parse::<u32>().unwrap();
                        i += 1;
                    }
                    _ => {
                        parsed_header_sections += 1;
                        i += 1;
                    }
                }
            }
        }
    }

    pub async fn load(config: &Config) -> Self {
        info!("Loading ARIN stats...");

        // Check if we need to redownload file
        match providers::load_provider_sources(config, "arin.stats") {
            Some(sources) => {
                sources.check_and_download().await;

                let mut stat_entries = Vec::new();
                for source in sources.iter() {
                    stat_entries.append(&mut StatsProvider::load_source(source).await);
                }

                info!("Loaded ARIN stats!");
                stat_entries.sort();

                StatsProvider {
                    values: stat_entries,
                    sources,
                }
            }
            None => panic!("Failed to load sources for ARIN stats!"),
        }
    }
}
