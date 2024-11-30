use recovered::RecoveredEntry;

use crate::utils::CIDR;

pub mod recovered;
pub mod reserved;

pub struct Providers {
    pub recovered: Vec<RecoveredEntry>,
    pub reserved: Vec<CIDR>,
}
