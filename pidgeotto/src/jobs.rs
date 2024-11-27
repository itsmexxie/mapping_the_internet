use std::net::Ipv4Addr;

use tokio::sync::mpsc;
use uuid::Uuid;

pub struct Job {
    pub id: Uuid,
    pub payload: JobPayload,
    pub priority: u16,
}

pub enum JobPayload {
    Query(Ipv4Addr),
    Online(Ipv4Addr),
}

pub enum JobQueueCommand {
    Add(Job),
}

pub async fn run(mut rx: mpsc::Receiver<JobQueueCommand>) {}
