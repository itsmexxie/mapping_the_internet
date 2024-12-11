use std::{path::Path, time::SystemTime};

use config::Config;
use reqwest::header::USER_AGENT;
use tokio::{
    fs::{self, File},
    io,
};
use tracing::{error, info};

use crate::get_config_value;

pub mod arin;
pub mod iana;
pub mod thyme;

pub struct Providers {
    pub arin: arin::Providers,
    pub iana: iana::Providers,
    pub thyme: thyme::Providers,
}

impl Providers {
    pub async fn load(config: &Config) -> Providers {
        Providers {
            arin: arin::Providers {
                stats: arin::stats::StatsProvider::load(&config).await,
            },
            iana: iana::Providers {
                recovered: iana::recovered::RecoveredProvider::load(&config).await,
                reserved: iana::reserved::ReservedProvider::load(&config).await,
            },
            thyme: thyme::Providers {
                asn: thyme::asn_prefixes::load(&config).await,
                rir: thyme::rir_allocations::load(&config).await,
            },
        }
    }

    pub async fn register_and_load(
        config: &Config,
        registered: &mut Vec<Vec<String>>,
    ) -> Providers {
        Providers {
            arin: arin::Providers {
                stats: arin::stats::StatsProvider::register_and_load(&config, registered).await,
            },
            iana: iana::Providers {
                recovered: iana::recovered::RecoveredProvider::register_and_load(
                    &config, registered,
                )
                .await,
                reserved: iana::reserved::ReservedProvider::load(&config).await,
            },
            thyme: thyme::Providers {
                asn: thyme::asn_prefixes::load(&config).await,
                rir: thyme::rir_allocations::load(&config).await,
            },
        }
    }
}

pub async fn download_file(config: &Config, section: &str) {
    let url = &get_config_value::<String>(config, &concat_string!(section, ".url"));
    let filepath_cnf = &get_config_value::<String>(config, &concat_string!(section, ".filepath"));
    let filepath = Path::new(filepath_cnf);

    if fs::metadata(filepath).await.is_err() {
        let prefix = filepath.parent().unwrap();
        fs::create_dir_all(prefix).await.unwrap();
    }

    let client = reqwest::Client::new();

    let file_in = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15")
		.send()
        .await
        .unwrap_or_else(|_| panic!("Failed to download file for section {}!", section))
        .bytes()
        .await
        .unwrap();
    let mut file_in_ref = file_in.as_ref();
    let mut file_out = File::create(filepath)
        .await
        .unwrap_or_else(|_| panic!("Failed to create file for section {}!", section));

    io::copy(&mut file_in_ref, &mut file_out)
        .await
        .unwrap_or_else(|_| panic!("Failed to write file for section {}!", section));

    info!("Downloaded file for section {}!", section);
}

pub async fn check_file(config: &Config, section: &str) -> bool {
    // Check if we need to download a fresh asn prefixes file
    let filepath_cnf = &get_config_value::<String>(config, &concat_string!(section, ".filepath"));
    let filepath = Path::new(filepath_cnf);
    let max_time = config
        .get_int(&concat_string!(section, ".max_time"))
        .unwrap_or(2592000); // Default 1 month expiration time

    if !filepath.exists() {
        info!("File for section {} missing!", section);
        false
    } else {
        let file_metadata = fs::metadata(filepath).await.unwrap();

        if let Ok(time_modified) = file_metadata.modified() {
            let time_now = SystemTime::now();
            if time_now.duration_since(time_modified).unwrap().as_secs() > max_time as u64 {
                info!("File for section {} stale!", section);
                false
            } else {
                info!("File for section {} OK!", section);
                true
            }
        } else {
            error!(
                "Couldn't read modified timestamp of file for section {}!",
                section
            );
            false
        }
    }
}

pub async fn check_and_download(config: &Config, section: &str) {
    if !check_file(config, &section).await {
        download_file(config, &section).await;
    }
}

pub async fn check_many_and_download(config: &Config, sections: &[&str]) {
    for section in sections {
        let section_str = concat_string!("providers.", section);
        if !check_file(config, &section_str).await {
            download_file(config, &section_str).await;
        }
    }
}
