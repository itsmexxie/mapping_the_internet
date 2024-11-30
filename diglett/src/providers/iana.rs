use crate::utils::CIDR;

pub mod reserved;

pub struct Providers {
    pub reserved: Vec<CIDR>,
}
