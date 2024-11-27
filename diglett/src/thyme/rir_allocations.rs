use std::{fmt::Display, path::Path, str::FromStr};

use config::Config;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{get_config_value, utils::CIDR};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RirAllocationEntry {
    pub prefix: CIDR,
    pub rir: String,
}

impl PartialOrd for RirAllocationEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.prefix.mask.partial_cmp(&self.prefix.mask)
    }
}

impl Ord for RirAllocationEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.prefix.mask.cmp(&self.prefix.mask)
    }
}

impl Display for RirAllocationEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "address: {}, mask: {}, rir: {}",
            self.prefix.prefix, self.prefix.mask, self.rir
        ))
    }
}

pub async fn load(config: &Config) -> Vec<RirAllocationEntry> {
    info!("Loading RIR allocations...");

    let rir_allocations_filepath_cnf =
        &get_config_value::<String>(config, "thyme.rir.allocations.filepath");
    let rir_allocations_filepath = Path::new(rir_allocations_filepath_cnf);

    let mut file: File = File::open(Path::new(rir_allocations_filepath))
        .await
        .unwrap();
    let mut contents_str = String::new();
    file.read_to_string(&mut contents_str).await.unwrap();

    let mut rir_allocations = Vec::new();

    let re = Regex::new(r"[\t ]+(\d+\/\d)[\t ]+(.+)").unwrap();
    for (_, [prefix, rir]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
        let parsed_cidr = CIDR::from_str(prefix).unwrap();
        rir_allocations.push(RirAllocationEntry {
            prefix: parsed_cidr,
            rir: rir.to_string(),
        });
    }

    rir_allocations
}
