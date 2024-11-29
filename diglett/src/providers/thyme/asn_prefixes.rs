use std::{fmt::Display, path::Path, str::FromStr, u32};

use config::Config;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{get_config_value, utils::CIDR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsnPrefixEntry {
    pub prefix: CIDR,
    pub asn: u32,
}

impl PartialOrd for AsnPrefixEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.prefix.mask.partial_cmp(&self.prefix.mask)
    }
}

impl Ord for AsnPrefixEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.prefix.mask.cmp(&self.prefix.mask)
    }
}

impl Display for AsnPrefixEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "address: {}, mask: {}, asn: {}",
            self.prefix.prefix, self.prefix.mask, self.asn
        ))
    }
}

pub async fn load(config: &Config) -> Vec<AsnPrefixEntry> {
    info!("Loading ASN prefixes...");

    let asn_prefixes_filepath_cnf =
        &get_config_value::<String>(config, "providers.thyme.asn_prefixes.filepath");
    let asn_prefixes_filepath = Path::new(asn_prefixes_filepath_cnf);

    let mut file = File::open(asn_prefixes_filepath).await.unwrap();
    let mut contents_str = String::new();
    file.read_to_string(&mut contents_str).await.unwrap();

    let mut prefixes = Vec::new();

    let re = Regex::new(r"([\d\.]+\/\d{1,2})[\t ]+(\d+)").unwrap();
    for (_, [prefix, asn]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
        let parsed_cidr = CIDR::from_str(prefix).unwrap();
        prefixes.push(AsnPrefixEntry {
            prefix: parsed_cidr,
            asn: asn.parse().unwrap(),
        });
    }

    info!("Loaded ASN prefixes!");

    prefixes.sort();
    prefixes
}
