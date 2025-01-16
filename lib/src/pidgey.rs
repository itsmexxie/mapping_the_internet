#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use uuid::Uuid;

use crate::types::{AllocationState, Rir};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PidgeyCommand {
    pub id: Uuid,
    pub payload: PidgeyCommandPayload,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PidgeyCommandPayload {
    Register,
    Deregister,
    Query { address: Ipv4Addr },
    AllocationState { address: Ipv4Addr },
    Rir { address: Ipv4Addr, top: bool },
    Autsys { address: Ipv4Addr },
    Country { address: Ipv4Addr },
    Online { address: Ipv4Addr },
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PidgeyCommandResponse {
    pub id: Uuid,
    pub payload: PidgeyCommandResponsePayload,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PidgeyCommandResponsePayload {
    Register {
        unit_uuid: Uuid,
    },
    Deregister,
    Query {
        allocation_state: AllocationState,
        top_rir: Option<Rir>,
        rir: Option<Rir>,
        autsys: Option<u32>,
        country: Option<String>,
        online: bool,
    },
    AllocationState {
        value: AllocationState,
    },
    Rir {
        value: Option<Rir>,
    },
    Autsys {
        value: Option<u32>,
    },
    Country {
        value: Option<String>,
    },
    Online {
        value: bool,
        reason: Option<String>,
    },
}
