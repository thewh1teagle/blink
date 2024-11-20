/*
cargo run --example simple
*/
fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    let interface = blinkscan::get_default_interface().unwrap();
    let network = blinkscan::create_network(&interface);
    for host in blinkscan::scan_network(network, std::time::Duration::from_secs(3)) {
        println!("{:?}", host);
    }
}
