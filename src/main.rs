mod scanner;
use std::time::Duration;
use clap::Parser;
use colored::Colorize;
use scanner::{get_default_interface,get_interfaces, create_network, scan_network, Host};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List network interfaces
    #[arg(short, long)]
    list: bool,

    /// Index for interface
    #[arg(short, long)]
    index: Option<u32>,

    /// Index for interface
    #[arg(short, long)]
    timeout: Option<humantime::Duration>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if args.list {
        let interfaces = get_interfaces();
        for interface in interfaces {
            println!("{}. {}", interface.index, interface.friendly_name.clone().unwrap_or(interface.name.clone()));
        }
        return;
    }
    
    let interface = if let Some(index) = args.index {
        let interfaces = get_interfaces();
        interfaces.iter().find(|i| i.index == index).unwrap().clone() // Cloning the interface found
    } else {
        get_default_interface().unwrap()
    };
    log::debug!("interface is {:?}", interface.clone().friendly_name.unwrap());
    let network = create_network(&interface);
    
    let timeout = args.timeout.map(|d| d.into()).unwrap_or(Duration::from_millis(600));
    let host_iterator = scan_network(network, timeout);
    
    for host in host_iterator {
        print_host(&host);
    }
}


fn print_host(host: &Host) {
    println!(
        "Host: {}{}",
        host.host.bold().green(),
        if let Some(hostname) = &host.hostname {
            format!(" ({})", hostname.bold())
        } else {
            String::new()
        }
    );
    if let Some(mac) = &host.mac {
        println!("  - MAC Address: {}", mac.bold());
    }
    if let Some(vendor) = &host.vendor {
        println!("  - Vendor: {}", vendor.bold());
    }
    println!();
}
