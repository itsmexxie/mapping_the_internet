use std::{
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use tokio::net::TcpStream;

pub trait ToIpv4Addrs {
    fn to_ipv4_address(&self) -> Result<Ipv4Addr, AddrParseError>;
}

impl ToIpv4Addrs for String {
    fn to_ipv4_address(&self) -> Result<Ipv4Addr, AddrParseError> {
        Ipv4Addr::from_str(self)
    }
}

impl ToIpv4Addrs for &String {
    fn to_ipv4_address(&self) -> Result<Ipv4Addr, AddrParseError> {
        Ipv4Addr::from_str(self)
    }
}

impl ToIpv4Addrs for str {
    fn to_ipv4_address(&self) -> Result<Ipv4Addr, AddrParseError> {
        Ipv4Addr::from_str(self)
    }
}

pub struct Gust(Ipv4Addr);

impl Gust {
    pub fn new<A: ToIpv4Addrs>(value: A) -> Result<Self, AddrParseError> {
        match value.to_ipv4_address() {
            Ok(address) => Ok(Gust(address)),
            Err(error) => Err(error),
        }
    }

    pub async fn attack(&self, port: u16, timeout: u32) -> bool {
        match tokio::time::timeout(Duration::from_secs(timeout.into()), async move {
            match TcpStream::connect(SocketAddr::new(IpAddr::V4(self.0), port)).await {
                Ok(_) => true,
                Err(_) => false,
            }
        })
        .await
        {
            Ok(value) => value,
            Err(_) => false,
        }
    }
}
