use std::{net::Ipv4Addr, sync::Arc};

use priority_queue::PriorityQueue;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Job {
    pub id: Uuid,
    pub payload: JobPayload,
    pub response: tokio::sync::oneshot::Sender<JobResponse>,
    pub priority: u16,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum JobPayload {
    Query(Ipv4Addr),
    Online(Ipv4Addr),
}

pub enum JobResponse {
    Online(bool),
}

pub enum JobQueueCommand {
    Add(Job),
}

pub async fn run(mut rx: mpsc::Receiver<JobQueueCommand>) {
    let job_queue = Arc::new(RwLock::new(PriorityQueue::<Job, u16>::new()));

    // Watch for incoming job payloads
    tokio::spawn(async move {
        match rx.recv().await {
            Some(job) => match job {
                JobQueueCommand::Add(job) => {
                    let priority = job.priority;
                    job_queue.write().await.push(job, priority);
                }
            },
            None => {}
        }
    });

    tokio::spawn(async move {})
}
