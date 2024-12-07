use std::{fmt::Display, path::Path, str::FromStr};

use config::Config;
use mtilib::types::Rir;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{get_config_value, providers, utils::CIDR};

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

pub async fn load(config: &Config) -> Vec<RirAllocationEntry> {
    info!("Loading RIR allocations...");

    // Check if we need to redownload file
    providers::check_many_and_download(config, &["thyme.rir_allocations"]).await;

    // Get the configured filepath and read the file into memory
    let rir_allocations_filepath_cnf =
        &get_config_value::<String>(config, "providers.thyme.rir_allocations.filepath");
    let rir_allocations_filepath = Path::new(rir_allocations_filepath_cnf);

    let mut file: File = File::open(rir_allocations_filepath).await.unwrap();
    let mut contents_str = String::new();
    file.read_to_string(&mut contents_str).await.unwrap();

    // Parse
    let mut rir_allocations = Vec::new();

    let re = Regex::new(r"[\t ]+(\d+\/\d)[\t ]+(.+)").unwrap();
    for (_, [prefix, rir]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
        let parsed_cidr = CIDR::from_str(prefix).unwrap();
        rir_allocations.push(RirAllocationEntry {
            cidr: parsed_cidr,
            rir: Rir::from_str(rir).unwrap(),
        });
    }

    info!("Loaded RIR allocations!");
    rir_allocations.sort();
    rir_allocations
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::utils::CIDR;

    use super::RirAllocationEntry;

    #[test]
    fn test_rir_allocation_entry_ord() {
        let entry_a = RirAllocationEntry {
            cidr: CIDR::from_str("1.0.0.0/8").unwrap(),
            rir: mtilib::types::Rir::Apnic,
        };

        let entry_b = RirAllocationEntry {
            cidr: CIDR::from_str("1.1.1.1/32").unwrap(),
            rir: mtilib::types::Rir::Apnic,
        };

        let mut list = vec![entry_a.clone(), entry_b.clone()];
        list.sort();
        assert_eq!(list, vec![entry_b, entry_a]);
    }
}
