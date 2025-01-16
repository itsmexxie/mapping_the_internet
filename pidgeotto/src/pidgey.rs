use std::collections::HashMap;

use mtilib::pidgey::{PidgeyCommand, PidgeyCommandResponsePayload};
use rand::seq::IteratorRandom;
use tokio::sync::{Notify, RwLock};
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PidgeyUnit {
    pub id: uuid::Uuid,
    pub tx: tokio::sync::mpsc::Sender<PidgeyUnitRequest>,
    pub available: bool,
}

impl PidgeyUnit {
    pub fn new(id: Uuid, tx: tokio::sync::mpsc::Sender<PidgeyUnitRequest>) -> PidgeyUnit {
        PidgeyUnit {
            id,
            tx,
            available: true,
        }
    }
}

#[derive(Debug)]
pub struct PidgeyUnitRequest {
    pub command: PidgeyCommand,
    pub response: tokio::sync::oneshot::Sender<PidgeyCommandResponsePayload>,
}

#[derive(Debug)]
pub struct Pidgey {
    pub units: RwLock<HashMap<Uuid, PidgeyUnit>>,
    pub unit_available: Notify,
}

impl Pidgey {
    pub fn new() -> Self {
        Pidgey {
            units: RwLock::new(HashMap::new()),
            unit_available: Notify::new(),
        }
    }

    pub async fn get_unit(&self) -> PidgeyUnit {
        loop {
            let lock = self.units.read().await;
            if lock.len() > 0 {
                if let Some(unit) = lock
                    .iter()
                    .filter(|x| x.1.available)
                    .choose(&mut rand::thread_rng())
                {
                    return unit.1.clone();
                }
            }
            drop(lock);
            self.unit_available.notified().await;
        }
    }

    pub async fn register_unit(&self, unit: PidgeyUnit) {
        info!("Registered unit {}", unit.id);
        self.units.write().await.insert(unit.id, unit);
        self.unit_available.notify_waiters();
    }

    pub async fn deregister_unit(&self, id: &Uuid) -> bool {
        match self.units.write().await.remove(&id) {
            Some(_) => {
                info!("Deregistered unit {}", id);
                return true;
            }
            None => {
                info!("Failed to deregister unit, no unit found! (id: {})", id);
                return false;
            }
        }
    }

    pub async fn is_registered(&self, id: &Uuid) -> bool {
        self.units.write().await.contains_key(id)
    }
}
