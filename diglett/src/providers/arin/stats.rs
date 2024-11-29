use std::{net::Ipv4Addr, path::Path, str::FromStr};

use axum::extract::ConnectInfo;
use config::Config;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{get_config_value, utils::CIDR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationState {
    Reserved,
    Unallocated,
    Allocated,
}

#[derive(Debug)]
pub enum AllocStateParseErr {
    UnknownState(String),
}

impl FromStr for AllocationState {
    type Err = AllocStateParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reserved" => Ok(AllocationState::Reserved),
            "available" => Ok(AllocationState::Unallocated),
            "allocated" => Ok(AllocationState::Allocated),
            "assigned" => Ok(AllocationState::Allocated), // Maybe not correct?
            _ => Err(AllocStateParseErr::UnknownState(s.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rir {
    Arin,
    RipeNcc,
    Apnic,
    Lacnic,
    Afrinic,
}

#[derive(Debug)]
pub enum RirParseErr {
    UnknownRir,
}

impl FromStr for Rir {
    type Err = RirParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arin" => Ok(Rir::Arin),
            "ripencc" => Ok(Rir::RipeNcc),
            "apnic" => Ok(Rir::Apnic),
            "lacnic" => Ok(Rir::Lacnic),
            "afrinic" => Ok(Rir::Afrinic),
            _ => Err(RirParseErr::UnknownRir),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatEntry {
    pub prefix: CIDR,
    pub allocation_state: AllocationState,
    pub rir: Rir,
    pub country: Option<String>,
}

impl PartialOrd for StatEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.prefix.mask.partial_cmp(&self.prefix.mask)
    }
}

impl Ord for StatEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.prefix.mask.cmp(&self.prefix.mask)
    }
}

pub async fn load(config: &Config) -> Vec<StatEntry> {
    info!("Loading ARIN stats...");

    let mut stat_entries = Vec::new();

    let sections = ["arin", "ripencc", "apnic", "lacnic", "afrinic"];

    for section in sections {
        let section_filepath_cnf = &get_config_value::<String>(
            config,
            &concat_string!("providers.arin.stats.", section, ".filepath"),
        );
        let section_filepath = Path::new(section_filepath_cnf);

        let mut file = File::open(section_filepath).await.unwrap();
        let mut contents_str = String::new();
        file.read_to_string(&mut contents_str).await.unwrap();
        let lines = contents_str.split("\n").collect::<Vec<_>>();

        let mut i = 1;
        let mut parsed_header = false;
        let mut ipv4_offset = 4;
        let mut ipv4_count = 0;
        loop {
            if lines[i].starts_with("#") {
                i += 1;
                continue;
            }

            if i >= ipv4_offset + ipv4_count {
                break;
            }

            if parsed_header {
                let parts = lines[i].split("|").collect::<Vec<_>>();
                let alloc_state = AllocationState::from_str(parts[6]).unwrap();
                let country = match alloc_state {
                    AllocationState::Reserved => None,
                    AllocationState::Unallocated => None,
                    AllocationState::Allocated => Some(parts[1].to_string()),
                };

                stat_entries.push(StatEntry {
                    prefix: CIDR {
                        prefix: Ipv4Addr::from_str(parts[3]).unwrap().into(),
                        mask: (u32::MAX & !(parts[4].parse::<u32>().unwrap() - 1)).count_ones()
                            as u16,
                    },
                    allocation_state: alloc_state,
                    rir: Rir::from_str(parts[0]).unwrap(),
                    country,
                });

                i += 1;
            } else {
                let parts = lines[i].split("|").collect::<Vec<_>>();
                match parts[2] {
                    "ipv4" => {
                        ipv4_count = parts[4].parse::<usize>().unwrap();
                        i = ipv4_offset;
                        parsed_header = true;
                    }
                    "ipv6" | "asn" => {
                        ipv4_offset += parts[4].parse::<usize>().unwrap();
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
    }

    stat_entries.sort();
    stat_entries
}
