use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

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

impl Display for CIDR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "prefix: {}, mask: {}",
            Ipv4Addr::from_bits(self.prefix),
            self.mask
        ))
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

impl PartialOrd for CIDR {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CIDR {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.mask.cmp(&other.mask)
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, net::Ipv4Addr, str::FromStr};

    use super::CIDR;

    #[test]
    fn test_address_cidr_check() {
        let prefix = CIDR::from_str("1.1.1.0/25").unwrap();
        assert!(prefix.address_is_in(Ipv4Addr::from_str("1.1.1.127").unwrap().into()));
    }

    #[test]
    fn test_cidr_ord() {
        let prefix_a = CIDR::from_str("1.1.1.1/32").unwrap();
        let prefix_b = CIDR::from_str("1.0.0.0/8").unwrap();
        assert_eq!(prefix_a.cmp(&prefix_b), Ordering::Greater)
    }
}
