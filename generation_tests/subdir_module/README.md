
When lib.rs declares `mod subdir;` pointing to subdir/mod.rs, the module file
should be included in the parent crate's srcs and NOT get its own rust_library
target in the subdirectory.

This tests that:
1. lib.rs crate roots discover modules in subdirectories.
2. Subdirectory BUILD files don't create rust rules for files claimed by a parent crate.
3. Unclaimed files in the subdirectory still get their own rules.
