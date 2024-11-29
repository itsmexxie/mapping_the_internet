use std::{net::Ipv4Addr, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CIDR {
    pub prefix: u32,
    pub mask: u16,
}

impl CIDR {
    pub fn new(prefix: u32, mask: u16) -> Self {
        CIDR { prefix, mask }
    }

    pub fn address_is_in(&self, address: u32) -> bool {
        let mask = u32::MAX << (32 - self.mask);
        (self.prefix & mask) == (address & mask)
    }
}

impl FromStr for CIDR {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("/").map(|s| s.to_string()).collect::<Vec<String>>();
        if parts[0].split(".").count() == 1 {
            parts[0] += ".0";
        }
        if parts[0].split(".").count() == 2 {
            parts[0] += ".0";
        }
        if parts[0].split(".").count() == 3 {
            parts[0] += ".0";
        }

        let parsed_address = match Ipv4Addr::from_str(&parts[0]) {
            Ok(address) => address,
            Err(_) => return Err("Failed to convert address to u32!"),
        };
        let parsed_mask = match parts[1].parse::<u16>() {
            Ok(mask) => mask,
            Err(_) => return Err("Failed to convert mask to u16!"),
        };

        Ok(CIDR {
            prefix: parsed_address.into(),
            mask: parsed_mask,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, str::FromStr};

    use super::CIDR;

    #[test]
    fn test_address_cidr_check() {
        let prefix = CIDR::from_str("1.1.1.0/25").unwrap();
        assert_eq!(
            prefix.address_is_in(Ipv4Addr::from_str("1.1.1.127").unwrap().into()),
            true
        );
    }
}
