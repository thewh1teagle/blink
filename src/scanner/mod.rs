use dns_lookup::lookup_addr;
use ipnetwork::Ipv4Network;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};
mod vendor;
use crossbeam_channel::{unbounded, Receiver};
pub use netdev::{get_default_interface, Interface};
use netneighbours::get_table;
use pinger::ping;

pub fn get_interfaces() -> Vec<Interface> {
    let mut interfaces = netdev::get_interfaces();
    if let Ok(default_interface) = get_default_interface() {
        // Sort interfaces prioritizing the default interface to be first
        interfaces.sort_by(|a, b| {
            if a.index == default_interface.index {
                std::cmp::Ordering::Less
            } else if b.index == default_interface.index {
                std::cmp::Ordering::Greater
            } else {
                a.index.cmp(&b.index)
            }
        });
    }
    interfaces
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Host {
    pub host: String,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub hostname: Option<String>,
}

type Table = Vec<(std::net::IpAddr, macaddr::MacAddr6)>;

pub struct HostIterator {
    pub rx: Receiver<Ipv4Addr>,
    pub vendor: vendor::Vendor,
    pub arp_table: Table,
}

impl Iterator for HostIterator {
    type Item = Host;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(active_ip) = self.rx.recv() {
            if let Some(info) = self.arp_table.iter().find(|i| i.0 == active_ip) {
                let mac = &info.1;
                let vendor_oui = vendor::get_vendor_oui(mac.as_bytes()).unwrap();
                let vendor_name = self
                    .vendor
                    .search_by_mac(&vendor_oui)
                    .unwrap_or_default()
                    .unwrap_or_default();
                let hostname = lookup_addr(&IpAddr::from(active_ip)).unwrap_or_default();
                tracing::debug!("hostanme is {}", hostname);
                let host = Host {
                    host: active_ip.to_string(),
                    hostname: Some(hostname),
                    mac: Some(mac.to_string()),
                    vendor: Some(vendor_name),
                };
                return Some(host);
            }
        }
        None
    }
}

pub fn create_network(interface: &Interface) -> Ipv4Network {
    tracing::debug!("getting gateway for {:?}", interface);
    let gateway = interface.clone().gateway.expect("No gateway");
    let gateway = gateway.ipv4.first().unwrap();
    let netmask = interface.ipv4.first().unwrap().netmask;
    tracing::debug!("gateway: {:?} netmask: {}", gateway, netmask);
    Ipv4Network::new(*gateway, ipv4_to_prefix(netmask)).unwrap()
}

fn ipv4_to_prefix(netmask: Ipv4Addr) -> u8 {
    netmask.octets().iter().map(|&b| b.count_ones() as u8).sum()
}

pub fn scan_network(network: Ipv4Network, timeout: Duration) -> HostIterator {
    if network.size() > 256 {
        tracing::warn!(
            "The network is larger than /24 (more than 255 IP addresses). This may take a while."
        );
    }

    let (tx, rx) = unbounded();
    let mut arp_table = get_table();
    arp_table.retain(|e| !e.1.is_nil());

    for ip in network.iter() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let stream = ping(ip.to_string(), None);
            if let Ok(stream) = stream {
                if stream.recv_timeout(timeout).is_ok() {
                    tx.send(ip).unwrap();
                }
            }
        });
    }

    HostIterator {
        rx,
        vendor: vendor::Vendor::default(),
        arp_table,
    }
}
