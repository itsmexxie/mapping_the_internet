use asn_prefixes::AsnPrefixesProvider;
use rir_allocations::RirAllocationsProvider;

pub mod asn_prefixes;
pub mod rir_allocations;

pub struct Providers {
    pub asn_prefixes: AsnPrefixesProvider,
    pub rir_allocations: RirAllocationsProvider,
}
