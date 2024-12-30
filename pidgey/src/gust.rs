use std::{
    collections::HashMap,
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use tokio::{net::TcpStream, sync::Mutex};

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

impl ToIpv4Addrs for Ipv4Addr {
    fn to_ipv4_address(&self) -> Result<Ipv4Addr, AddrParseError> {
        Ok(*self)
    }
}

#[derive(Clone)]
pub struct Gust(Ipv4Addr);

impl Gust {
    pub fn new<A: ToIpv4Addrs>(value: A) -> Result<Self, AddrParseError> {
        match value.to_ipv4_address() {
            Ok(address) => Ok(Gust(address)),
            Err(error) => Err(error),
        }
    }

    pub async fn attack_range(
        &self,
        range: std::ops::RangeInclusive<u16>,
        timeout: u32,
    ) -> Result<HashMap<u16, bool>, &'static str> {
        // Values from different ports
        // This is stored in a hashmap because we theoretically want to query nonconsecutive ports
        let ports = Arc::new(Mutex::new(HashMap::new()));
        let mut port_tasks = Vec::new();

        // Try the ports parallely
        for port in range {
            let cloned_ports = ports.clone();
            let cloned_gust = self.clone();
            port_tasks.push(tokio::spawn(async move {
                let result = cloned_gust.attack(port, timeout).await;

                if result {
                    cloned_ports.lock().await.insert(port, result);
                }
            }));
        }

        for port_task in port_tasks {
            port_task.await.unwrap();
        }

        match Arc::try_unwrap(ports) {
            Ok(ports) => Ok(ports.into_inner()),
            Err(_) => Err("Failed to unwrap arc"),
        }
    }

    pub async fn attack(&self, port: u16, timeout: u32) -> bool {
        (tokio::time::timeout(Duration::from_secs(timeout.into()), async move {
            TcpStream::connect(SocketAddr::new(IpAddr::V4(self.0), port))
                .await
                .is_ok()
        })
        .await).unwrap_or_default()
    }
}
