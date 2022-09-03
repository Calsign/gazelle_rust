use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut imports = rust_parser::parse_imports(args.file)?;
    imports.sort();

    println!("Imports:");
    for import in imports {
        println!("  {}", import);
    }
    Ok(())
}
