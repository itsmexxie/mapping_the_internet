use std::{path::Path, time::SystemTime};

use config::Config;
use reqwest::header::USER_AGENT;
use tokio::{
    fs::{self, File},
    io,
};
use tracing::{debug, error};

pub mod arin;
pub mod iana;
pub mod thyme;

pub struct ProviderSource {
    pub filepath: String,
    pub url: String,
    pub max_time: u64,
}

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
                asn: thyme::asn_prefixes::AsnPrefixesProvider::load(&config).await,
                rir: thyme::rir_allocations::RirAllocationsProvider::load(&config).await,
            },
        }
    }

    // pub async fn register_and_load(
    //     config: &Config,
    //     registered: &mut Vec<Vec<String>>,
    // ) -> Providers {
    //     Providers {
    //         arin: arin::Providers {
    //             stats: arin::stats::StatsProvider::register_and_load(&config, registered).await,
    //         },
    //         iana: iana::Providers {
    //             recovered: iana::recovered::RecoveredProvider::register_and_load(
    //                 &config, registered,
    //             )
    //             .await,
    //             reserved: iana::reserved::ReservedProvider::load(&config).await,
    //         },
    //         thyme: thyme::Providers {
    //             asn: thyme::asn_prefixes::load(&config).await,
    //             rir: thyme::rir_allocations::load(&config).await,
    //         },
    //     }
    // }
}

pub async fn download_file(url: &str, filepath: &str) {
    let filepath = Path::new(filepath);

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
        .unwrap_or_else(|_| panic!("Failed to download file {}!", filepath.to_str().unwrap()))
        .bytes()
        .await
        .unwrap();
    let mut file_in_ref = file_in.as_ref();
    let mut file_out = File::create(filepath)
        .await
        .unwrap_or_else(|_| panic!("Failed to create file {}!", filepath.to_str().unwrap()));

    io::copy(&mut file_in_ref, &mut file_out)
        .await
        .unwrap_or_else(|_| panic!("Failed to write file {}!", filepath.to_str().unwrap()));
}

// max_time default 2592000
pub async fn check_file(filepath: &str, max_time: u64) -> bool {
    // Check if we need to download a fresh asn prefixes file
    let filepath = Path::new(filepath);

    if !filepath.exists() {
        debug!("File {} missing!", filepath.to_str().unwrap());
        false
    } else {
        let file_metadata = fs::metadata(filepath).await.unwrap();

        if let Ok(time_modified) = file_metadata.modified() {
            let time_now = SystemTime::now();
            if time_now.duration_since(time_modified).unwrap().as_secs() > max_time as u64 {
                debug!("File {} stale!", filepath.to_str().unwrap());
                false
            } else {
                debug!("File {} OK!", filepath.to_str().unwrap());
                true
            }
        } else {
            error!(
                "Couldn't read modified timestamp of file {}!",
                filepath.to_str().unwrap()
            );
            false
        }
    }
}

pub fn load_provider_sources(config: &Config, provider: &str) -> Option<Vec<ProviderSource>> {
    match config.get_array(&concat_string!("providers.", provider, ".sources")) {
        Ok(sources) => {
            let mut parsed_sources = Vec::new();

            for source in sources {
                let source_map = source
                    .into_table()
                    .expect("Invalid config (provider source must be a TOML table)!");
                let filepath = source_map
                    .get("filepath")
                    .expect("Invalid config (provider source must have a filepath set)!")
                    .to_owned()
                    .into_string()
                    .unwrap();
                let url = source_map
                    .get("url")
                    .expect("Invalid config (provider source must have a url set)!")
                    .to_owned()
                    .into_string()
                    .unwrap();
                let max_time = source_map
                    .get("max_time")
                    .unwrap_or(&config::Value::from(2592000))
                    .to_owned()
                    .into_int()
                    .expect(
                        "Invalid config (max_time for provider source must be a valid integer)!",
                    ) as u64;

                parsed_sources.push(ProviderSource {
                    filepath,
                    url,
                    max_time,
                });
            }
            Some(parsed_sources)
        }
        Err(_) => None,
    }
}

pub async fn check_and_download(sources: &Vec<ProviderSource>) {
    for source in sources {
        if !check_file(&source.filepath, source.max_time).await {
            download_file(&source.url, &source.filepath).await;
        }
    }
}
