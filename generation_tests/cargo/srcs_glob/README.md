Test for rust_srcs_glob directive.

This test verifies that when rust_srcs_glob is enabled, the plugin generates
srcs = glob(["src/**/*.rs"], exclude = ["src/main.rs"]) for rust_library targets
instead of listing files explicitly.

Note: The glob is only applied to rust_library and rust_proc_macro targets.
rust_binary and cargo_build_script targets continue to list files explicitly.
The top-level main.rs file is excluded from the glob since it should only appear
in rust_binary targets.
