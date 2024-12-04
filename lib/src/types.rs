use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationState {
    Unknown,
    Reserved,
    Unallocated,
    Allocated,
}

impl AllocationState {
    pub fn id(&self) -> String {
        match self {
            AllocationState::Unknown => String::from("unknown"),
            AllocationState::Reserved => String::from("reserved"),
            AllocationState::Unallocated => String::from("unallocated"),
            AllocationState::Allocated => String::from("allocated"),
        }
    }
}

impl Display for AllocationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AllocationState::Unknown => "Unknown",
            AllocationState::Reserved => "Reserved",
            AllocationState::Unallocated => "Unallocated",
            AllocationState::Allocated => "Allocated",
        })
    }
}

#[derive(Debug)]
pub enum AllocStateParseErr {
    UnknownState(String),
}

impl FromStr for AllocationState {
    type Err = AllocStateParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reserved" => Ok(AllocationState::Reserved),
            "available" => Ok(AllocationState::Unallocated),
            "allocated" => Ok(AllocationState::Allocated),
            "assigned" => Ok(AllocationState::Allocated), // Maybe not correct?
            _ => Err(AllocStateParseErr::UnknownState(s.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rir {
    Unknown,
    Arin,
    Ripencc,
    Apnic,
    Lacnic,
    Afrinic,
    Other,
}

impl Rir {
    pub fn id(&self) -> String {
        match self {
            Rir::Unknown => String::from("unknown"),
            Rir::Arin => String::from("arin"),
            Rir::Ripencc => String::from("ripencc"),
            Rir::Apnic => String::from("apnic"),
            Rir::Lacnic => String::from("lacnic"),
            Rir::Afrinic => String::from("afrinic"),
            Rir::Other => String::from("other"),
        }
    }
}

impl Display for Rir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Rir::Unknown => "Unknown",
            Rir::Arin => "ARIN",
            Rir::Ripencc => "RIPE NCC",
            Rir::Apnic => "APNIC",
            Rir::Lacnic => "LACNIC",
            Rir::Afrinic => "AfriNIC",
            Rir::Other => "Other",
        })
    }
}

#[derive(Debug)]
pub enum RirParseErr {
    UnknownRir(String),
}

impl FromStr for Rir {
    type Err = RirParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "arin" => Ok(Rir::Arin),
            "ripencc" => Ok(Rir::Ripencc),
            "ripe ncc" => Ok(Rir::Ripencc),
            "apnic" => Ok(Rir::Apnic),
            "lacnic" => Ok(Rir::Lacnic),
            "afrinic" => Ok(Rir::Afrinic),
            _ => Err(RirParseErr::UnknownRir(s.to_string())),
        }
    }
}
