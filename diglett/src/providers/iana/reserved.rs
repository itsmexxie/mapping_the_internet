use std::{path::Path, str::FromStr};

use config::Config;
use tracing::info;

use crate::{get_config_value, utils::CIDR};

pub async fn load(config: &Config) -> Vec<CIDR> {
    info!("Loading IANA reserved blocks...");

    // Check if we need to redownload file
    crate::providers::check_many_and_download(config, &["iana.reserved"]).await;

    // Get the configured filepath and read the file into memory
    let section_filepath_cnf =
        &get_config_value::<String>(config, "providers.iana.reserved.filepath");
    let section_filepath = Path::new(section_filepath_cnf);

    let mut reader = csv::Reader::from_reader(std::fs::File::open(section_filepath).unwrap());

    // Parse
    let mut reserved_blocks = Vec::new();

    for reader_line in reader.deserialize() {
        let addresses: String = reader_line.unwrap();

        for address in addresses.split(",").map(|x| x.trim()) {
            let address = address.split(" ").collect::<Vec<_>>()[0];
            reserved_blocks.push(CIDR::from_str(address).unwrap());
        }
    }

    // Manually add multicast block
    // TODO: Move into its own section (create a new allocation state)
    reserved_blocks.push(CIDR::from_str("224.0.0.0/4").unwrap());

    info!("Loaded IANA reserved blocks!");
    reserved_blocks.sort();
    reserved_blocks
}
