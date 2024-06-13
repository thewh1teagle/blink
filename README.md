# Blink

Blink is a lightweight, fast, and cross-platform network scanning tool that operates without requiring admin privileges.

# Features

- 🚀 Fast: thanks to rust, blink scan 255 targets in 1-2 seconds.
- 🖥️ Cross-platform: Works seamlessly on macOS, Windows, and Linux.
- 🔒 No admin rights required: Run Blink without needing administrative permissions.
- ℹ️ Useful information: Provides details such as host vendor names, IP addresses, MAC addresses, and even hostnames.

# Installation

For installation instructions, visit the [Blink Website](https://thewh1teagle.github.io/blink/).

# Build

```console
cargo build --release
```

# Usage

```console
blink --help
```

# Use as a library

1. Install `blinkscan`

```console
cargo add blinkscan
```

2. Include in `main.rs`

```rust
fn main() {
    let interface = blinkscan::get_default_interface().unwrap();
    let network = blinkscan::create_network(&interface);
    for host in blinkscan::scan_network(network, std::time::Duration::from_secs(3)) {
        println!("{:?}", host);
    }
}
```

# GUI

See [thewh1teagle/Ruscan](https://github.com/thewh1teagle/Ruscan)
