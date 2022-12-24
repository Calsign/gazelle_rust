#![deny(unused_must_use)]

use std::error::Error;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;

use protobuf::{CodedInputStream, CodedOutputStream, RepeatedField};

use messages_rust_proto::{
    LockfileCratesRequest, LockfileCratesRequest_oneof_lockfile, LockfileCratesResponse, Request,
    Request_oneof_kind, RustImportsRequest, RustImportsResponse,
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
            response.set_success(true);
            response.set_hints(rust_imports.hints);
            response.imports = RepeatedField::from_vec(rust_imports.imports);
            response.test_imports = RepeatedField::from_vec(rust_imports.test_imports);
        }
        Err(err) => {
            // Don't crash gazelle if we encounter an error, instead bubble it up so that we can
            // report it and keep going.
            // TODO: It's possible that some errors here actually should be fatal.
            response.set_success(false);
            response.set_error_msg(err.to_string());
        }
    }

    Ok(response)
}

fn handle_lockfile_crates_request(
    request: LockfileCratesRequest,
) -> Result<LockfileCratesResponse, Box<dyn Error>> {
    let crates = match request.lockfile.unwrap() {
        LockfileCratesRequest_oneof_lockfile::lockfile_path(path) => {
            lockfile_crates::get_bazel_lockfile_crates(PathBuf::from(path))?
        }
        LockfileCratesRequest_oneof_lockfile::cargo_lockfile_path(path) => {
            lockfile_crates::get_cargo_lockfile_crates(PathBuf::from(path))?
        }
    };

    let mut response = LockfileCratesResponse::default();
    response.set_crates(RepeatedField::from_vec(crates));

    Ok(response)
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
            let mut writer = std::io::stdout();
            // TODO: avoid opening two stdout handles
            let mut writer2 = std::io::stdout();
            let mut stdout = CodedOutputStream::new(&mut writer);

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
                let size = CodedInputStream::from_bytes(&buf[..SF32]).read_uint32()? as usize;
                if size > buf.len() {
                    // grow buffer as needed
                    buf = vec![0; size];
                }

                stdin.read_exact(&mut buf[..size])?;
                let request: Request = protobuf::parse_from_bytes(&buf[..size])?;

                if let Some(kind) = request.kind {
                    let response: Box<dyn protobuf::Message> = match kind {
                        Request_oneof_kind::rust_imports(request) => {
                            Box::new(handle_rust_imports_request(request)?)
                        }
                        Request_oneof_kind::lockfile_crates(request) => {
                            Box::new(handle_lockfile_crates_request(request)?)
                        }
                    };

                    stdout.write_fixed32_no_tag(response.compute_size())?;
                    response.write_to(&mut stdout)?;
                    stdout.flush()?;
                    // need to flush the underlying stdout because protobuf doesn't do that for us
                    writer2.flush()?;
                }
            }
        }
    }

    Ok(())
}
