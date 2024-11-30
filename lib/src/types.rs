use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationState {
    Unknown,
    Reserved,
    Unallocated,
    Allocated,
}

#[derive(Debug)]
pub enum AllocStateParseErr {
    UnknownState(String),
}

impl FromStr for AllocationState {
    type Err = AllocStateParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reserved" => Ok(AllocationState::Reserved),
            "available" => Ok(AllocationState::Unallocated),
            "allocated" => Ok(AllocationState::Allocated),
            "assigned" => Ok(AllocationState::Allocated), // Maybe not correct?
            _ => Err(AllocStateParseErr::UnknownState(s.to_string())),
        }
    }
}

impl ToString for AllocationState {
    fn to_string(&self) -> String {
        match self {
            AllocationState::Unknown => String::from("unknown"),
            AllocationState::Reserved => String::from("reserved"),
            AllocationState::Unallocated => String::from("unallocated"),
            AllocationState::Allocated => String::from("allocated"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rir {
    Arin,
    RipeNcc,
    Apnic,
    Lacnic,
    Afrinic,
}

#[derive(Debug)]
pub enum RirParseErr {
    UnknownRir(String),
}

impl FromStr for Rir {
    type Err = RirParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arin" => Ok(Rir::Arin),
            "ripencc" => Ok(Rir::RipeNcc),
            "apnic" => Ok(Rir::Apnic),
            "lacnic" => Ok(Rir::Lacnic),
            "afrinic" => Ok(Rir::Afrinic),
            _ => Err(RirParseErr::UnknownRir(s.to_string())),
        }
    }
}

impl ToString for Rir {
    fn to_string(&self) -> String {
        match self {
            Rir::Arin => String::from("ARIN"),
            Rir::RipeNcc => String::from("RIPE NCC"),
            Rir::Apnic => String::from("APNIC"),
            Rir::Lacnic => String::from("LACNIC"),
            Rir::Afrinic => String::from("AfriNIC"),
        }
    }
}
