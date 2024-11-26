use std::{path::Path, time::SystemTime};

use config::Config;
use tokio::{
    fs::{self, File},
    io,
};
use tracing::{error, info};

pub mod asn_prefixes;
pub mod rir_allocations;

use crate::get_config_value;

pub async fn download_file(config: &Config, section: &str) {
    let url = get_config_value::<String>(config, &concat_string!(section, ".url"));
    let filepath_cnf = &get_config_value::<String>(config, &concat_string!(section, ".filepath"));
    let filepath = Path::new(filepath_cnf);

    if !fs::metadata(filepath).await.is_ok() {
        let prefix = filepath.parent().unwrap();
        fs::create_dir_all(prefix).await.unwrap();
    }

    let file_in = reqwest::get(&url)
        .await
        .expect(&format!("Failed to download file for section {}!", section))
        .bytes()
        .await
        .unwrap();
    let mut file_in_ref = file_in.as_ref();
    let mut file_out = File::create(filepath)
        .await
        .expect(&format!("Failed to create file for section {}!", section));

    io::copy(&mut file_in_ref, &mut file_out)
        .await
        .expect(&format!("Failed to write file for section {}!", section));

    info!("Downloaded file for section {}!", section);
}

pub async fn check_file(config: &Config, section: &str) -> bool {
    // Check if we need to download a fresh asn prefixes file
    let filepath_cnf = &get_config_value::<String>(config, &concat_string!(section, ".filepath"));
    let filepath = Path::new(filepath_cnf);
    let max_time = get_config_value::<i64>(config, &concat_string!(section, ".max_time"));

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
