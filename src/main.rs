use std::{net::{IpAddr, Ipv4Addr}, time::Duration};
use clap::Parser;
use dns_lookup::lookup_addr;
use ipnetwork::Ipv4Network;
use serde::{Serialize, Deserialize};
mod vendor;
use colored::Colorize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List network interfaces
    #[arg(short, long)]
    list: bool,

    /// Index for interface
    #[arg(short, long)]
    index: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Host {
    host: String,
    mac: Option<String>,
    vendor: Option<String>,
    hostname: Option<String>
}


fn main() {
    env_logger::init();
    let args = Args::parse();
    if args.list {
        let mut interfaces = netdev::get_interfaces();
        interfaces.sort_by_key(|i| i.index);
        for interface in interfaces {
            println!("{}. {}", interface.index, interface.friendly_name.clone().unwrap_or(interface.name.clone()));
        }
        return;
    }
    
    let interface = if let Some(index) = args.index {
        log::debug!("search interface {}", index);
        let interfaces = netdev::get_interfaces();
        log::debug!("interfaces: {:?}", interfaces);
        interfaces.iter().find(|i| i.index == index).unwrap().clone() // Cloning the interface found
    } else {
        netdev::get_default_interface().unwrap()
    };
    log::debug!("interface is {:?}", interface.friendly_name.unwrap());
    let gateway = interface.gateway.unwrap();
    

    let gateway = gateway.ipv4.first().unwrap();
    let netmask = interface.ipv4.first().unwrap().netmask;

    log::debug!("gateway: {:?} netmask: {}", gateway, netmask);

    let network = Ipv4Network::new(*gateway, ipv4_to_prefix(netmask)).unwrap();
    log::debug!("network: {}", network);
    scan_network(network);
}

fn ipv4_to_prefix(netmask: Ipv4Addr) -> u8 {
    netmask.octets().iter().map(|&b| b.count_ones() as u8).sum()
}

fn scan_network(network: Ipv4Network) {

    if network.size() > 256 {
        log::warn!("The network is larger than /24 (more than 255 IP addresses). This may take a while.");
    }
    let (tx, rx) = crossbeam_channel::unbounded();

    for ip in network.iter() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let stream = pinger::ping(ip.to_string(), None);
            let result = stream.unwrap().recv_timeout(Duration::from_millis(500));
            if result.is_ok() {
                tx.send(ip).unwrap();
            }
            
        });
    }

    drop(tx);

    let mut vendor = vendor::Vendor::new();
    let mut hosts: Vec<Host> = Vec::new();
    let mut arp_table = netneighbours::get_table();
    arp_table = arp_table.iter().filter(|e| !e.1.is_nil()).cloned().collect();
    while let Ok(active_ip) = rx.recv() {
        // println!("Active IP: {}", active_ip);
        if let Some(info) = arp_table.iter().find(|i| i.0 == active_ip) {
            let mac = info.1;
            let vendor_oui = vendor::get_vendor_oui(mac.as_bytes()).unwrap();
            let vendor_name = vendor.search_by_mac(&vendor_oui).unwrap_or_default().unwrap_or_default();
            let hostname = lookup_addr(&IpAddr::from(active_ip)).unwrap_or_default();
            let host = Host{host: active_ip.to_string(), hostname: Some(hostname), mac: Some(mac.to_string()), vendor: Some(vendor_name)};
            print_host(&host);
            hosts.push(host);
        }
    }
    log::debug!("finish");
}

fn print_host(host: &Host) {
    println!(
        "Host: {}{}",
        host.host.bold().green(), // Bold and green
        if let Some(hostname) = &host.hostname {
            format!(" ({})", hostname.bold()) // Bold hostname in parentheses
        } else {
            String::new()
        }
    );
    if let Some(mac) = &host.mac {
        println!("  - MAC Address: {}", mac.bold()); // Bold MAC address
    }
    if let Some(vendor) = &host.vendor {
        println!("  - Vendor: {}", vendor.bold()); // Bold vendor
    }
    println!(); // Empty line between hosts
}