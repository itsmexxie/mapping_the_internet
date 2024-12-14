use std::{path::Path, time::SystemTime};

use config::Config;
use reqwest::header::USER_AGENT;
use stats::StatsProvider;
use tokio::{
    fs::{self, File},
    io,
};
use tracing::{debug, error};

pub mod iana;
pub mod stats;
pub mod thyme;

pub struct ProviderSource {
    pub filepath: String,
    pub url: String,
    pub max_time: u32,
}

impl ProviderSource {
    pub async fn download(&self) {
        let filepath = Path::new(&self.filepath);

        if fs::metadata(filepath).await.is_err() {
            let prefix = filepath.parent().unwrap();
            fs::create_dir_all(prefix).await.unwrap();
        }

        let client = reqwest::Client::new();

        let file_in = client
			.get(&self.url)
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

    pub async fn check(&self) -> bool {
        // Check if we need to download a fresh asn prefixes file
        let filepath = Path::new(&self.filepath);

        if !filepath.exists() {
            debug!("File {} missing!", filepath.to_str().unwrap());
            false
        } else {
            let file_metadata = fs::metadata(filepath).await.unwrap();

            if let Ok(time_modified) = file_metadata.modified() {
                let time_now = SystemTime::now();
                if time_now.duration_since(time_modified).unwrap().as_secs() > self.max_time as u64
                {
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
}

pub trait CheckAndDownloadSource {
    fn check_and_download(&self) -> impl std::future::Future<Output = ()> + Send;
}

impl CheckAndDownloadSource for ProviderSource {
    async fn check_and_download(&self) {
        if !self.check().await {
            self.download().await;
        }
    }
}

impl CheckAndDownloadSource for Vec<ProviderSource> {
    async fn check_and_download(&self) {
        for source in self {
            source.check_and_download().await;
        }
    }
}

// #[derive(Clone)]
// pub enum Provider<'a> {
//     StatsProvider(&'a StatsProvider),
//     IanaReservedProvider(&'a iana::reserved::ReservedProvider),
//     IanaRecoveredProvider(&'a iana::recovered::RecoveredProvider),
//     ThymeAsnPrefixesProvider(&'a thyme::asn_prefixes::AsnPrefixesProvider),
//     ThymeRirAllocationsProvider(&'a thyme::rir_allocations::RirAllocationsProvider),
// }

// impl<'a> Provider<'a> {
//     pub fn sources(&self) -> &Vec<ProviderSource> {
//         match self {
//             Provider::StatsProvider(stats_provider) => &stats_provider.sources,
//             Provider::IanaReservedProvider(reserved_provider) => &reserved_provider.sources,
//             Provider::IanaRecoveredProvider(recovered_provider) => &recovered_provider.sources,
//             Provider::ThymeAsnPrefixesProvider(asn_prefixes_provider) => {
//                 &asn_prefixes_provider.sources
//             }
//             Provider::ThymeRirAllocationsProvider(rir_allocations_provider) => {
//                 &rir_allocations_provider.sources
//             }
//         }
//     }
// }

pub struct Providers {
    pub stats: StatsProvider,
    pub iana: iana::Providers,
    pub thyme: thyme::Providers,
}

impl Providers {
    pub async fn load(config: &Config) -> Self {
        Providers {
            stats: StatsProvider::load(&config).await,
            iana: iana::Providers {
                reserved: iana::reserved::ReservedProvider::load(&config).await,
                recovered: iana::recovered::RecoveredProvider::load(&config).await,
            },
            thyme: thyme::Providers {
                asn_prefixes: thyme::asn_prefixes::AsnPrefixesProvider::load(&config).await,
                rir_allocations: thyme::rir_allocations::RirAllocationsProvider::load(&config)
                    .await,
            },
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
                    .unwrap_or(&config::Value::from(2592000)) // default 1 month
                    .to_owned()
                    .into_int()
                    .expect(
                        "Invalid config (max_time for provider source must be a valid integer)!",
                    ) as u32;

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
