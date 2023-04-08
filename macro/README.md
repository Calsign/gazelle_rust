
This is a tiny proc_macro crate that provides attribute macros for gazelle. The macros return their
inputs unchanged, and exist only to pass information to gazelle. The attributes are also referred to
as "directives".

An alternative would be for gazelle to parse special comments. Using the attribute is nicer in some
ways: it allows more direct control over which items should be affected by a particular directive,
and it's easier to integrate into the existing syn-based parser.

This crate has a cargo-compatible directory structure so that in the future it can be published on
crates.io to support projects that build with both bazel and cargo.
