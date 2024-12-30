#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod allocation_state;
pub mod rir;

pub use allocation_state::AllocationState;
pub use rir::Rir;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValueResponse<T> {
    pub value: T,
}
