use asn_prefixes::AsnPrefixEntry;
use rir_allocations::RirAllocationEntry;

pub mod asn_prefixes;
pub mod rir_allocations;

pub struct Providers {
    pub asn: Vec<AsnPrefixEntry>,
    pub rir: Vec<RirAllocationEntry>,
}
