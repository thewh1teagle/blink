use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;

fn main() {
    // The URL to download the file from
    let url = "https://standards-oui.ieee.org/oui/oui.csv";
    
    // Path to save the downloaded file
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("src").join("oui.csv");

    if dest_path.exists() {
        return;
    }
    // Create the destination file
    let mut dest_file = File::create(&dest_path).expect("Failed to create file");

    // Download the file and copy its contents to the destination file
    let response = reqwest::blocking::get(url).expect("Failed to download file");
    let content = response.bytes().expect("Failed to read response bytes");

    copy(&mut content.as_ref(), &mut dest_file).expect("Failed to copy content to file");

    println!("cargo:rerun-if-changed=build.rs");
}
