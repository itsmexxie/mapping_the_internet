use std::collections::HashMap;

use mtilib::pidgey::PidgeyCommand;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct PidgeyUnit {
    pub id: uuid::Uuid,
    pub tx: tokio::sync::mpsc::Sender<PidgeyCommand>,
    pub available: bool,
}

impl PidgeyUnit {
    pub fn new(id: Uuid, tx: tokio::sync::mpsc::Sender<PidgeyCommand>) -> PidgeyUnit {
        PidgeyUnit {
            id,
            tx,
            available: false,
        }
    }
}

#[derive(Debug)]
pub struct Pidgey {
    pub units: HashMap<Uuid, PidgeyUnit>,
}

impl Pidgey {
    pub fn new() -> Self {
        Pidgey {
            units: HashMap::new(),
        }
    }

    pub fn register_unit(&mut self, unit: PidgeyUnit) {
        debug!("Registered unit {}", unit.id);
        self.units.insert(unit.id, unit);
    }

    pub fn deregister_unit(&mut self, id: &Uuid) -> bool {
        match self.units.remove(id) {
            Some(_) => {
                debug!("Deregistered unit {}", id);
                return true;
            }
            None => {
                debug!("Failed to deregister unit, no unit found! (id: {})", id);
                return false;
            }
        }
    }

    pub fn is_registered(&self, id: &Uuid) -> bool {
        self.units.contains_key(id)
    }
}
