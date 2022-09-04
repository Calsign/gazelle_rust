#![deny(unused_must_use)]

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

pub fn get_lockfile_crates(lockfile_path: PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
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
            // TODO: support crates with names that don't match the target name
            // TODO: support proc_macros
            crates.push(dep.target.clone());
        }
    }

    Ok(crates)
}
