use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lib.rs");

    let contents = r#"
pub fn hello() -> &'static str {
    "Hello, world!"
}
"#;

    fs::write(dest_path, contents).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
