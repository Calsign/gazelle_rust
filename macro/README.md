gazelle\_rust is a gazelle language plugin for Rust; automatic dependency management for projects
built with Bazel.

References:
 - [gazelle\_rust](https://github.com/Calsign/gazelle_rust)
 - [Gazelle](https://github.com/bazelbuild/bazel-gazelle)
 - [rules\_rust](https://github.com/bazelbuild/rules_rust)
 - [Bazel](https://bazel.build/)

This is a tiny proc\_macro crate that provides attribute macros for gazelle. The macros return their
inputs unchanged, and exist only to pass information to gazelle. The attributes are also referred to
as "directives".

An alternative would be for gazelle to parse special comments. Using the attribute is nicer in some
ways: it allows more direct control over which items should be affected by a particular directive,
and it's easier to integrate into the existing syn-based parser.

This crate has a cargo-compatible directory because it is also published to crates.io to support
projects that build both with bazel and cargo.

Since crates.io has a global namespace, this is the one and only "gazelle" package. Please reach out
to me if you're working on an alternative gazelle plugin for Rust, I'm happy to make changes as long
as backwards-compatibility is maintained.
