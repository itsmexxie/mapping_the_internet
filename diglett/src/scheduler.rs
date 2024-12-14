// use std::{sync::Arc, time::Duration};

// use clokwerk::{AsyncScheduler, TimeUnits};
// use tokio::sync::{Mutex, RwLock};
// use tracing::info;

// use crate::providers::{CheckAndDownloadSource, Providers};

// pub async fn run(providers: Arc<RwLock<Providers>>) {
//     let scheduler = Arc::new(Mutex::new(AsyncScheduler::new()));

//     // Main scheduler loop
//     let cloned_scheduler = scheduler.clone();
//     tokio::spawn(async move {
//         loop {
//             cloned_scheduler.lock().await.run_pending().await;
//             tokio::time::sleep(Duration::from_millis(1000)).await;
//         }
//     });

//     let locked_providers = providers.clone().write().await;
//     for provider in locked_providers.list() {
//         let cloned_providers = providers.clone();
//         for source in provider.sources() {
//             scheduler
//                 .lock()
//                 .await
//                 .every((source.max_time + 120).seconds()) // We want to wait a bit longer so that the check fails if the file is stale and the download + load cycle takes a few seconds
//                 .run(move || {
//                     let cloned_provider = provider.clone();
//                     let cloned_source = source.clone();
//                     let cloned_providers = cloned_providers.clone();
//                     async move {
//                         cloned_source.check_and_download().await;
//                     }
//                 });
//         }
//     }
// }
