#!/usr/bin/env python3
"""
This is a standalone script for bootstrapping a bazel workspace in an existing cargo project and
configuring it to use gazelle_rust.

This is highly experimental and likely won't work for any non-trivial project. Use at your own risk.
"""

import argparse
import glob
import os
import subprocess
import sys
import typing as T


GAZELLE_RUST_COMMIT = "04e5450054ba5c89013022ad14c50b68c05214fd"
GAZELLE_RUST_SHA256 = "41b9261187aeb6a6e0d097ebbcd5e10cf89c439d950b9398d5bdc10abf614ab5"

RULES_RUST_VERSION = "0.40.0"
RULES_RUST_SHA256 = "c30dfdf1e86fd50650a76ea645b3a45f2f00667b06187a685e9554e167ca97ee"

RUST_VERSION = "1.73.0"

BAZEL_VERSION = "6.4.0"


def write_workspace(args: argparse.Namespace) -> None:
    if os.path.exists("WORKSPACE"):
        print("WORKSPACE already exists, skipping setup")
        return

    print("Writing WORKSPACE...")

    with open("WORKSPACE", "w") as workspace:
        workspace.write(
            """load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
"""
        )

        if args.local_gazelle_rust:
            workspace.write(
                """
# Load gazelle_rust from a local directory (bootstrap_from_cargo.py was run
# with --local-gazelle-rust).
local_repository(
    name = "gazelle_rust",
    path = "{path}",
)
""".format(
                    path=args.local_gazelle_rust
                )
            )
        else:
            workspace.write(
                """
http_archive(
    name = "gazelle_rust",
    sha256 = "{sha256}",
    strip_prefix = "gazelle_rust-{commit}",
    urls = ["https://github.com/Calsign/gazelle_rust/archive/{commit}.zip"],
)
""".format(
                    commit=args.gazelle_rust_commit,
                    sha256=args.gazelle_rust_sha256,
                ),
            )

        workspace.write(
            """
http_archive(
    name = "rules_rust",
    # This patch is currently necessary for gazelle_rust to parse crate_universe lockfiles.
    sha256 = "{sha256}",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz"],
)
""".format(
                version=args.rules_rust_version,
                sha256=args.rules_rust_sha256,
            )
        )

        workspace.write(
            """
load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    versions = ["{version}"],
)
""".format(
                version=args.rust_version
            )
        )

        if not args.skip_crate_universe:
            manifest_strs = [
                " " * 8 + '"' + manifest + '"' + "," for manifest in get_manifests()
            ]
            manifests_str = "\n".join(manifest_strs)

            workspace.write(
                """
load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

# Use crate_universe to pull in external crates using the same lockfile that cargo uses.
crates_repository(
    name = "crates",
    lockfile = "//:cargo-bazel-lock.json",
    cargo_lockfile = "//:Cargo.lock",
    manifests = [
{manifests}
    ],
)

load("@crates//:defs.bzl", "crate_repositories")

crate_repositories()
""".format(
                    manifests=manifests_str
                )
            )

        workspace.write(
            """
# Load dependencies for gazelle_rust.

load("@gazelle_rust//:deps1.bzl", "gazelle_rust_dependencies1")

gazelle_rust_dependencies1()

load("@gazelle_rust//:deps2.bzl", "gazelle_rust_dependencies2")

gazelle_rust_dependencies2()
"""
        )


def get_manifests() -> T.List[str]:
    manifests = []
    for root, dirs, files in os.walk(os.getcwd(), followlinks=False):
        dirs[:] = [d for d in dirs if d not in ("target", ".git")]

        for f in files:
            if f == "Cargo.toml":
                package = os.path.relpath(root, os.getcwd())
                if package.startswith("."):
                    package = package[2:]

                # need to create build files so that these are valid labels
                build_file_path = os.path.join(package, "BUILD.bazel")
                if not os.path.exists(build_file_path) and package != "":
                    with open(build_file_path, "w") as build_file:
                        build_file.write("")

                manifests.append("//{}:{}".format(package, f))

    assert len(manifests) > 0, "did not find Cargo.toml"

    return manifests


def write_build(args: argparse.Namespace) -> None:
    if os.path.exists("BUILD.bazel"):
        print("BUILD.bazel already exists, skipping setup")
        return

    print("Writing BUILD.bazel...")

    with open("BUILD.bazel", "w") as build:
        build.write(
            """load("@bazel_gazelle//:def.bzl", "gazelle")
"""
        )

        if not args.skip_crate_universe:
            build.write(
                """
# Tell gazelle_rust to generate from Cargo.toml files rather than the
# default "pure-bazel" mode.
# gazelle:rust_mode generate_from_cargo

# Tell gazelle_rust where we get our external crates from.
# gazelle:rust_lockfile cargo-bazel-lock.json
# gazelle:rust_crates_prefix @crates//:
            """
            )

        build.write(
            """
# Gazelle target. Run with: bazel run //:gazelle
gazelle(
    name = "gazelle",
    gazelle = "@gazelle_rust//:gazelle_bin",
)
"""
        )


def write_lockfile(args: argparse.Namespace) -> None:
    if not args.skip_crate_universe:
        if os.path.exists("cargo-bazel-lock.json"):
            print("cargo-bazel-lock.json already exists, skipping setup")
            return

        # just touch the file, it will get pinned when we fetch later
        with open("cargo-bazel-lock.json", "w") as lockfile:
            lockfile.write("")


def write_gitignore(args: argparse.Namespace) -> None:
    print("Appending to .gitignore...")

    with open(".gitignore", "a") as gitignore:
        gitignore.write(
            """
# bazel output symlinks
/bazel-*
"""
        )


def write_bazelversion(args: argparse.Namespace) -> None:
    if os.path.exists(".bazelversion"):
        print(".bazelversion already exists, skipping setup")
        return

    print("Writing .bazelversion")

    with open(".bazelversion", "w") as bazelversion:
        bazelversion.write(BAZEL_VERSION)
        bazelversion.write("\n")


def fetch_repos(args: argparse.Namespace) -> None:
    if args.skip_crate_universe:
        print("Fetching bazel repositories...")
        env = {}
    else:
        print("Fetching bazel repositories and pinning crate_universe lockfile...")
        env = {"CARGO_BAZEL_REPIN": "workspace"}

    run_command(["bazel", "fetch", "//..."], env=env)


def run_gazelle(args: argparse.Namespace) -> None:
    print("Running gazelle to create bazel targets...")

    run_command(["bazel", "run", "//:gazelle"])


def build_and_test(args: argparse.Namespace) -> None:
    print("Building all bazel targets and running all tests...")

    run_command(["bazel", "test", "//..."])


def run_command(cmd: T.List[str], env: T.Dict[str, str] = {}) -> None:
    env_str = " ".join("{}={}".format(k, v) for k, v in env.items())
    print("Running: {}{}".format(env_str + " " if env_str else "", " ".join(cmd)))

    env_copy = os.environ.copy()
    env_copy.update(**env)

    try:
        subprocess.check_call(cmd, env=env_copy)
        print("\n")
    except subprocess.CalledProcessError as e:
        print("Command failed with exit code {}: {}".format(e.returncode, e))
        print("\nPlease try again manually after fixing the issues.")
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Set up a bazel workspace in an existing cargo project"
    )
    parser.add_argument(
        "--repo-root", default=os.getcwd(), help="directory containing cargo project"
    )
    parser.add_argument(
        "--local-gazelle-rust",
        help="if set, uses a local_repository for gazelle_rust at the given path",
    )
    parser.add_argument(
        "--gazelle-rust-commit",
        help="if set, changes the default gazelle_rust commit",
        default=GAZELLE_RUST_COMMIT,
    )
    parser.add_argument(
        "--gazelle-rust-sha256",
        help="if set, changes the default gazelle_rust sha256",
        default=GAZELLE_RUST_SHA256,
    )
    parser.add_argument(
        "--rules-rust-version",
        help="if set, changes the default rules_rust version",
        default=RULES_RUST_VERSION,
    )
    parser.add_argument(
        "--rules-rust-sha256",
        help="if set, changes the default rules_rust sha256",
        default=RULES_RUST_SHA256,
    )
    parser.add_argument(
        "--rust-version",
        help="if set, changes the default rust version",
        default=RUST_VERSION,
    )
    parser.add_argument(
        "--skip-crate-universe",
        action="store_true",
        help="skip setting up crate universe",
    )
    parser.add_argument(
        "--skip-initialize",
        action="store_true",
        help="don't run bazel, just write files",
    )

    args = parser.parse_args()

    if args.local_gazelle_rust:
        args.local_gazelle_rust = os.path.abspath(args.local_gazelle_rust)

    os.chdir(args.repo_root)

    write_workspace(args)
    write_build(args)
    write_lockfile(args)
    write_gitignore(args)
    write_bazelversion(args)

    if not args.skip_initialize:
        fetch_repos(args)
        run_gazelle(args)
        build_and_test(args)


if __name__ == "__main__":
    main()
