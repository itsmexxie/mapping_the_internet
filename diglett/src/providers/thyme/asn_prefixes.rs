use std::{fmt::Display, path::Path, str::FromStr};

use config::Config;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

use crate::{providers, utils::CIDR};

pub struct AsnPrefixesProvider {
    pub value: Vec<AsnPrefixEntry>,
}

impl AsnPrefixesProvider {
    pub async fn load(config: &Config) -> Self {
        info!("Loading ASN prefixes...");

        // Check if we need to redownload file
        match providers::load_provider_sources(config, "thyme.asn_prefixes") {
            Some(sources) => {
                providers::check_and_download(&sources).await;

                let mut prefixes = Vec::new();
                for source in sources {
                    let asn_prefixes_filepath = Path::new(&source.filepath);

                    let mut file = File::open(asn_prefixes_filepath).await.unwrap();
                    let mut contents_str = String::new();
                    file.read_to_string(&mut contents_str).await.unwrap();

                    let re = Regex::new(r"([\d\.]+\/\d{1,2})[\t ]+(\d+)").unwrap();
                    for (_, [prefix, asn]) in re.captures_iter(&contents_str).map(|c| c.extract()) {
                        let parsed_cidr = CIDR::from_str(prefix).unwrap();
                        prefixes.push(AsnPrefixEntry {
                            cidr: parsed_cidr,
                            asn: asn.parse().unwrap(),
                        });
                    }
                }

                info!("Loaded ASN prefixes!");
                prefixes.sort();

                AsnPrefixesProvider { value: prefixes }
            }
            None => panic!("Failed to load sources for Thyme's ASN prefixes"),
        }
    }
}

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
