use std::{sync::Arc, time::Duration};

use clokwerk::AsyncScheduler;
use tokio::sync::{Mutex, RwLock};

use crate::providers::Providers;

pub enum SchedulerCommand {
    CheckAndDownload(String),
}

pub async fn run(
    scheduler: Arc<Mutex<AsyncScheduler>>,
    rx: &mut tokio::sync::mpsc::Receiver<SchedulerCommand>,
    providers: Arc<RwLock<Providers>>,
) {
    // Job loop task
    tokio::spawn(async move {
        loop {
            scheduler.lock().await.run_pending().await;
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });

    // Watch for incoming job commands
    while let Some(command) = rx.recv().await {}
}
