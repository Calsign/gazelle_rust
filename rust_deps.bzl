load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository")

def rust_dependencies():
    crates_repository(
        name = "crates",
        lockfile = "//:cargo.bazel.lock",
        cargo_lockfile = "//:cargo.lock",
        packages = {
            "syn": crate.spec(
                version = "1.0",
                features = ["full", "visit", "extra-traits"],
            ),
            "clap": crate.spec(
                version = "3.2",
                features = ["derive"],
            ),
            "lazy_static": crate.spec(
                version = "1.4",
            ),
        },
    )
