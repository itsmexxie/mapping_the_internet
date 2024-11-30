use std::{fmt::Display, path::Path, str::FromStr};

use config::Config;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{get_config_value, providers, utils::CIDR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsnPrefixEntry {
    pub cidr: CIDR,
    pub asn: u32,
}

impl PartialOrd for AsnPrefixEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AsnPrefixEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cidr.cmp(&self.cidr)
    }
}

impl Display for AsnPrefixEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "address: {}, mask: {}, asn: {}",
            self.cidr.prefix, self.cidr.mask, self.asn
        ))
    }
}

pub async fn load(config: &Config) -> Vec<AsnPrefixEntry> {
    info!("Loading ASN prefixes...");

    // Check if we need to redownload file
    providers::check_many_and_download(config, &["thyme.asn_prefixes"]).await;

    // Get the configured filepath and read the file into memory
    let asn_prefixes_filepath_cnf =
        &get_config_value::<String>(config, "providers.thyme.asn_prefixes.filepath");
    let asn_prefixes_filepath = Path::new(asn_prefixes_filepath_cnf);

    let mut file = File::open(asn_prefixes_filepath).await.unwrap();
    let mut contents_str = String::new();
    file.read_to_string(&mut contents_str).await.unwrap();

    // Parse
    let mut prefixes = Vec::new();

    let re = Regex::new(r"([\d\.]+\/\d{1,2})[\t ]+(\d+)").unwrap();
    for (_, [prefix, asn]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
        let parsed_cidr = CIDR::from_str(prefix).unwrap();
        prefixes.push(AsnPrefixEntry {
            cidr: parsed_cidr,
            asn: asn.parse().unwrap(),
        });
    }

    info!("Loaded ASN prefixes!");
    prefixes.sort();
    prefixes
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::utils::CIDR;

    use super::AsnPrefixEntry;

    #[test]
    fn test_rir_allocation_entry_ord() {
        let entry_a = AsnPrefixEntry {
            cidr: CIDR::from_str("1.0.0.0/8").unwrap(),
            asn: 1,
        };

        let entry_b = AsnPrefixEntry {
            cidr: CIDR::from_str("1.1.1.1/32").unwrap(),
            asn: 1,
        };

        let mut list = vec![entry_a.clone(), entry_b.clone()];
        list.sort();
        assert_eq!(list, vec![entry_b, entry_a]);
    }
}
