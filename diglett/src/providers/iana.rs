use recovered::RecoveredProvider;
use reserved::ReservedProvider;

pub mod recovered;
pub mod reserved;

pub struct Providers {
    pub reserved: ReservedProvider,
    pub recovered: RecoveredProvider,
}
