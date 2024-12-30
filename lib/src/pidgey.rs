#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::Ipv4Addr};
use uuid::Uuid;

use crate::types::{AllocationState, Rir};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PidgeyCommand {
    Register,
    Deregister,
    Query {
        id: Uuid,
        address: Ipv4Addr,
        ports_start: Option<u16>,
        ports_end: Option<u16>,
    },
    AllocationState {
        id: Uuid,
        address: Ipv4Addr,
    },
    Rir {
        id: Uuid,
        address: Ipv4Addr,
        top: bool,
    },
    Asn {
        id: Uuid,
        address: Ipv4Addr,
    },
    Country {
        id: Uuid,
        address: Ipv4Addr,
    },
    Online {
        id: Uuid,
        address: Ipv4Addr,
    },
    Port {
        id: Uuid,
        address: Ipv4Addr,
        port: u16,
    },
    PortRange {
        id: Uuid,
        address: Ipv4Addr,
        start: Option<u16>,
        end: Option<u16>,
    },
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PidgeyCommandResponse {
    Register,
    Deregister,
    Query {
        id: Uuid,
        allocation_state: AllocationState,
        top_rir: Option<Rir>,
        rir: Option<Rir>,
        asn: Option<u32>,
        country: Option<String>,
        online: bool,
        ports: Option<HashMap<u16, bool>>,
    },
    AllocationState {
        id: Uuid,
        value: AllocationState,
    },
    Rir {
        id: Uuid,
        value: Option<Rir>,
    },
    Asn {
        id: Uuid,
        value: Option<u32>,
    },
    Country {
        id: Uuid,
        value: Option<String>,
    },
    Online {
        id: Uuid,
        value: bool,
        reason: Option<String>,
    },
    Port {
        id: Uuid,
        value: bool,
    },
    PortRange {
        id: Uuid,
        value: HashMap<u16, bool>,
    },
}
