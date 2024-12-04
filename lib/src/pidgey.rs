use std::{collections::HashMap, net::Ipv4Addr};

use serde::{Deserialize, Serialize};

use crate::types::Rir;

#[derive(Debug, Deserialize, Serialize)]
pub enum PidgeyCommand {
    Register,
    Deregister,
    AllocationState {
        address: Ipv4Addr,
    },
    AllocationStateRes {
        value: String,
    },
    Rir {
        address: Ipv4Addr,
        top: bool,
    },
    RirRes {
        value: Option<Rir>,
    },
    Asn {
        address: Ipv4Addr,
    },
    AsnRes {
        value: Option<u32>,
    },
    Country {
        address: Ipv4Addr,
    },
    CountryRes {
        value: Option<String>,
    },
    Online {
        address: Ipv4Addr,
    },
    OnlineRes {
        value: bool,
        reason: Option<String>,
    },
    PortRange {
        address: Ipv4Addr,
        start: Option<u16>,
        end: Option<u16>,
    },
    PortRangeRes {
        value: HashMap<u16, bool>,
    },
    Port {
        address: Ipv4Addr,
        port: u16,
    },
    PortRes {
        value: bool,
    },
}
