#![deny(unused_must_use)]

use std::error::Error;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use prost::Message;

use messages_proto::{
    CargoCrateInfo, CargoTomlRequest, CargoTomlResponse, Hints, LockfileCratesRequest,
    LockfileCratesResponse, Request, RustImportsRequest, RustImportsResponse,
    lockfile_crates_request, request,
};

#[derive(clap::Parser)]
enum Args {
    OneShot { path: PathBuf },
    StreamProto,
}

fn handle_rust_imports_request(
    request: RustImportsRequest,
) -> Result<RustImportsResponse, Box<dyn Error>> {
    let rust_imports = parser::parse_imports(PathBuf::from(request.file_path));

    let mut response = RustImportsResponse::default();
    match rust_imports {
        Ok(rust_imports) => {
            let hints = Hints {
                has_main: rust_imports.hints.has_main,
                has_test: rust_imports.hints.has_test,
                has_proc_macro: rust_imports.hints.has_proc_macro,
            };

            response.success = true;
            response.hints = Some(hints);
            response.imports = rust_imports.imports;
            response.test_imports = rust_imports.test_imports;
            response.extern_mods = rust_imports.extern_mods;
        }
        Err(err) => {
            // Don't crash gazelle if we encounter an error, instead bubble it up so that we can
            // report it and keep going.
            // TODO: It's possible that some errors here actually should be fatal.
            response.success = false;
            response.error_msg = err.to_string();
        }
    }

    Ok(response)
}

fn handle_lockfile_crates_request(
    request: LockfileCratesRequest,
) -> Result<LockfileCratesResponse, Box<dyn Error>> {
    let crates = match request.lockfile {
        Some(lockfile_crates_request::Lockfile::LockfilePath(path)) => {
            lockfile_crates::get_bazel_lockfile_crates(PathBuf::from(path))?
        }
        Some(lockfile_crates_request::Lockfile::CargoLockfilePath(path)) => {
            lockfile_crates::get_cargo_lockfile_crates(PathBuf::from(path))?
        }
        None => return Err("No lockfile path provided".into()),
    };

    Ok(LockfileCratesResponse { crates })
}

fn build_crate_info(product: cargo_toml::Product) -> CargoCrateInfo {
    let mut crate_info = CargoCrateInfo::default();

    if let Some(name) = product.name {
        crate_info.name = name;
    }
    if let Some(path) = product.path {
        let normalized_path = path.strip_prefix("./").unwrap_or(&path).to_string();
        crate_info.srcs = vec![normalized_path];
    }
    crate_info.proc_macro = product.proc_macro;

    crate_info
}

fn handle_cargo_toml_request(
    request: CargoTomlRequest,
) -> Result<CargoTomlResponse, Box<dyn Error>> {
    let mut manifest = cargo_toml::Manifest::from_path(&request.file_path)?;
    manifest.complete_from_path(&PathBuf::from(&request.file_path))?;

    let name = manifest.package.map(|p| p.name).unwrap_or_default();
    let library = manifest.lib.map(build_crate_info);
    let binaries = manifest.bin.into_iter().map(build_crate_info).collect();
    let tests = manifest.test.into_iter().map(build_crate_info).collect();
    let benches = manifest.bench.into_iter().map(build_crate_info).collect();
    let examples = manifest.example.into_iter().map(build_crate_info).collect();

    Ok(CargoTomlResponse {
        success: true,
        name,
        library,
        binaries,
        tests,
        benches,
        examples,
        error_msg: String::new(),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args {
        Args::OneShot { path } => {
            let mut rust_imports = parser::parse_imports(path)?;
            rust_imports.imports.sort();

            println!("Imports:");
            for import in rust_imports.imports {
                println!("  {}", import);
            }
        }
        Args::StreamProto => {
            let mut stdin = std::io::stdin();
            let mut stdout = std::io::stdout();

            let mut buf: Vec<u8> = vec![0; 1024];
            const SF32: usize = std::mem::size_of::<u32>();

            loop {
                match stdin.read_exact(&mut buf[..SF32]) {
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                        // EOF: parent process finished
                        break;
                    }
                    res => res?,
                }
                let size = i32::from_le_bytes(buf[..SF32].try_into()?) as usize;
                if size > buf.len() {
                    // grow buffer as needed
                    buf = vec![0; size];
                }

                stdin.read_exact(&mut buf[..size])?;
                let request = Request::decode(&buf[..size])?;

                if let Some(kind) = request.kind {
                    let response_bytes: Vec<u8> = match kind {
                        request::Kind::RustImports(request) => {
                            handle_rust_imports_request(request)?.encode_to_vec()
                        }
                        request::Kind::LockfileCrates(request) => {
                            handle_lockfile_crates_request(request)?.encode_to_vec()
                        }
                        request::Kind::CargoToml(request) => {
                            handle_cargo_toml_request(request)?.encode_to_vec()
                        }
                    };

                    let size_bytes = (response_bytes.len() as u32).to_le_bytes();
                    stdout.write_all(&size_bytes)?;
                    stdout.write_all(&response_bytes)?;
                    stdout.flush()?;
                }
            }
        }
    }

    Ok(())
}
