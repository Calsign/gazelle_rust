#![deny(unused_must_use)]

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use messages_rust_proto::Package;

pub fn get_bazel_lockfile_crates(lockfile_path: PathBuf) -> Result<Vec<Package>, Box<dyn Error>> {
    let lockfile = match File::open(&lockfile_path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "Could not find lockfile: {}",
                &lockfile_path.to_str().unwrap_or("<utf-8 decode error>")
            );
            std::process::exit(1);
        }
        file => file?,
    };
    let context: cargo_bazel::context::Context = match serde_json::from_reader(lockfile) {
        Err(err) => {
            eprintln!(
                "Could not parse lockfile {}: {}",
                &lockfile_path.to_str().unwrap_or("<utf-8 decode error>"),
                err
            );
            std::process::exit(1);
        }
        file => file?,
    };

    let mut crates = Vec::new();

    for workspace_member in context.workspace_members.keys() {
        let workspace_crate = context
            .crates
            .get(workspace_member)
            .expect("missing workspace member");

        for dep in workspace_crate.common_attrs.deps.get_iter(None).unwrap() {
            let mut package = Package::default();
            package.name = dep.target.clone();
            package.crate_name = cargo_bazel::utils::sanitize_module_name(&package.name);
            // TODO: support proc_macros
            package.proc_macro = false;

            crates.push(package);
        }
    }

    Ok(crates)
}

pub fn get_cargo_lockfile_crates(lockfile_path: PathBuf) -> Result<Vec<Package>, Box<dyn Error>> {
    let lockfile = match cargo_lock::Lockfile::load(&lockfile_path) {
        Err(err) => {
            eprintln!(
                "Could not load cargo lockfile {}: {}",
                lockfile_path.to_str().unwrap_or("<utf-8 decode error>"),
                err
            );
            std::process::exit(1);
        }
        file => file?,
    };

    let mut crates = Vec::new();

    for pkg in lockfile.packages {
        let mut package = Package::default();
        package.name = pkg.name.as_str().to_string();
        package.crate_name = cargo_bazel::utils::sanitize_module_name(&package.name);
        // TODO: support proc_macros
        package.proc_macro = false;

        crates.push(package);
    }

    Ok(crates)
}
