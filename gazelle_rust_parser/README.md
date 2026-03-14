gazelle\_rust is a gazelle language plugin for Rust; automatic dependency management for projects
built with Bazel.

References:
 - [gazelle\_rust](https://github.com/Calsign/gazelle_rust)
 - [Gazelle](https://github.com/bazelbuild/bazel-gazelle)
 - [rules\_rust](https://github.com/bazelbuild/rules_rust)
 - [Bazel](https://bazel.build/)

This crate is the core parser implementation used by gazelle\_rust for extracting imports from rust
sources. This is published as a standalone crate so that it may be reused for other tools, such as a
tool for automatic cargo dependency management.
