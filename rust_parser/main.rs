#![deny(unused_must_use)]

use std::error::Error;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;

use protobuf::{CodedInputStream, CodedOutputStream, RepeatedField};

use messages_rust_proto::{
    LockfileCratesRequest, LockfileCratesResponse, Request, Request_oneof_kind, RustImportsRequest,
    RustImportsResponse,
};

#[derive(clap::Parser)]
enum Args {
    OneShot { path: PathBuf },
    StreamProto,
}

fn handle_rust_imports_request(
    request: RustImportsRequest,
) -> Result<RustImportsResponse, Box<dyn Error>> {
    let imports = parser::parse_imports(PathBuf::from(request.file_path))?;

    let mut response = RustImportsResponse::default();
    response.imports = RepeatedField::from_vec(imports);

    Ok(response)
}

fn handle_lockfile_crates_request(
    request: LockfileCratesRequest,
) -> Result<LockfileCratesResponse, Box<dyn Error>> {
    let crates = lockfile_crates::get_lockfile_crates(PathBuf::from(request.lockfile_path))?;

    let mut response = LockfileCratesResponse::default();
    response.crates = RepeatedField::from_vec(crates);

    Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args {
        Args::OneShot { path } => {
            let mut imports = parser::parse_imports(path)?;
            imports.sort();

            println!("Imports:");
            for import in imports {
                println!("  {}", import);
            }
        }
        Args::StreamProto => {
            let mut stdin = std::io::stdin();
            let mut writer = std::io::stdout();
            // TODO: avoid opening two stdout handles
            let mut writer2 = std::io::stdout();
            let mut stdout = CodedOutputStream::new(&mut writer);

            const MAX_MSG_SIZE: usize = 4096;
            let mut buf: [u8; MAX_MSG_SIZE] = [0; MAX_MSG_SIZE];
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
                assert!(
                    size < MAX_MSG_SIZE,
                    "message size {} exceeds max message size {}",
                    size,
                    MAX_MSG_SIZE
                );

                stdin.read_exact(&mut buf[..size])?;
                let request: Request = protobuf::parse_from_bytes(&buf[..size])?;

                let response: Box<dyn protobuf::Message> =
                    match request.kind.expect("missing request kind") {
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

    Ok(())
}
