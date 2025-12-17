#![deny(unused_must_use)]

use std::error::Error;
use std::path::PathBuf;

use cargo_bazel::api::lockfile::CargoBazelLockfile;
use messages_proto::{Package, PackageDependency};

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
            let package = Package {
                name: crate_.name().to_string(),
                crate_name: library_target_name.to_string(),
                proc_macro: is_proc_macro,
                version: crate_.version().to_string(),
                workspace_member: false,
                dependencies: Vec::new(),
            };

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

pub fn make_package_dependency(dep: &cargo_lock::Dependency) -> PackageDependency {
    PackageDependency {
        name: dep.name.as_str().to_string(),
        version: dep.version.to_string(),
    }
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

    let mut crates = Vec::new();

    for pkg in lockfile.packages {
        if !is_workspace_target(pkg.name.as_str()) {
            let name = pkg.name.as_str().to_string();
            let package = Package {
                name: name.clone(),
                crate_name: name.replace('-', "_"),
                proc_macro: pkg
                    .dependencies
                    .iter()
                    .any(|dep| is_proc_macro_dep(dep.name.as_str())),
                version: pkg.version.to_string(),
                workspace_member: pkg.source.is_none(),
                dependencies: pkg
                    .dependencies
                    .into_iter()
                    .map(|dep| make_package_dependency(&dep))
                    .collect(),
            };

            crates.push(package);
        }
    }

    Ok(crates)
}
