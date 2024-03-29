#![deny(unused_must_use)]

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use cargo_bazel::api::lockfile::CargoBazelLockfile;
use messages_rust_proto::Package;

pub fn get_bazel_lockfile_crates(lockfile_path: PathBuf) -> Result<Vec<Package>, Box<dyn Error>> {
    let context = match cargo_bazel::api::lockfile::parse(&lockfile_path) {
        Err(err) => {
            eprintln!(
                "Could not parse lockfile {}: {}",
                &lockfile_path.to_str().unwrap_or("<utf-8 decode error>"),
                err,
            );
            std::process::exit(1);
        }
        file => file?,
    };

    let mut crates = Vec::new();

    let mut add_crate = |id: &_, is_proc_macro| {
        let crate_ = context.crate_info(id).expect("missing crate");

        if let Some(library_target_name) = &crate_.library_target_name() {
            let mut package = Package::default();
            package.set_name(crate_.name().to_string());
            package.set_crate_name(library_target_name.to_string());
            package.set_proc_macro(is_proc_macro);

            crates.push(package);
        }
    };

    for workspace_member in context.workspace_members() {
        let workspace_crate = context
            .crate_info(&workspace_member)
            .expect("missing workspace member");

        for dep in workspace_crate.normal_deps().values() {
            add_crate(&dep.id, false);
        }

        for dep in workspace_crate.dev_deps().values() {
            add_crate(&dep.id, false);
        }

        for proc_macro_dep in workspace_crate.proc_macro_deps().values() {
            add_crate(&proc_macro_dep.id, true);
        }

        for proc_macro_dep in workspace_crate.proc_macro_dev_deps().values() {
            add_crate(&proc_macro_dep.id, true);
        }
    }

    Ok(crates)
}

pub fn is_workspace_target(name: &str) -> bool {
    name == "direct-cargo-bazel-deps"
}

/// Cargo lockfiles don't indicate whether a crate is a proc_macro, so we guess. If a crate depends
/// on proc_macro or proc_macro2, it is almost certainly a proc_macro.
pub fn is_proc_macro_dep(name: &str) -> bool {
    name == "proc-macro" || name == "proc-macro2"
}

pub fn get_cargo_lockfile_crates(lockfile_path: PathBuf) -> Result<Vec<Package>, Box<dyn Error>> {
    let lockfile = match cargo_lock::Lockfile::load(&lockfile_path) {
        Err(err) => {
            eprintln!(
                "Could not load cargo lockfile {}: {}",
                lockfile_path.to_str().unwrap_or("<utf-8 decode error>"),
                err,
            );
            std::process::exit(1);
        }
        file => file?,
    };

    let mut is_proc_macro = HashMap::new();
    let mut deps = Vec::new();

    for pkg in lockfile.packages {
        if is_workspace_target(pkg.name.as_str()) {
            deps.extend(pkg.dependencies);
        } else {
            is_proc_macro.insert(
                pkg.name.as_str().to_string(),
                pkg.dependencies
                    .iter()
                    .any(|dep| is_proc_macro_dep(dep.name.as_str())),
            );
        }
    }

    let mut crates = Vec::new();

    for dep in deps {
        let mut package = Package::default();
        package.name = dep.name.as_str().to_string();
        package.crate_name = package.name.replace('-', "_");
        package.proc_macro = is_proc_macro[dep.name.as_str()];

        crates.push(package);
    }

    Ok(crates)
}
