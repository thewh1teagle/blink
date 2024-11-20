use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;

static OUT_FILENAME: &str = "oui.csv";
static OUI_URL: &str = "https://standards-oui.ieee.org/oui/oui.csv";
static OUI_FALLBACK_URL: &str =
    "https://github.com/thewh1teagle/blink/releases/download/v0.1.0/oui-2024-11-20.csv";

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("src").join(OUT_FILENAME);

    if dest_path.exists() {
        return;
    }
    let mut dest_file = File::create(&dest_path).expect("Failed to create file");
    let response = reqwest::blocking::get(OUI_URL)
        .or_else(|_| {
            println!("cargo:warning=Failed to download oui.csv. trying fallback url...");
            reqwest::blocking::get(OUI_FALLBACK_URL)
        })
        .expect("Failed to download file");
    let content = response.bytes().expect("Failed to read response bytes");
    copy(&mut content.as_ref(), &mut dest_file).expect("Failed to copy content to file");

    println!("cargo:rerun-if-changed=build.rs");
}
