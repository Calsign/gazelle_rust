
Tests the adjacent file module style (introduced in Rust 2018) where modules use
parent.rs instead of parent/mod.rs.

Structure:
- lib.rs declares `mod parent;`
- parent.rs (adjacent file style) declares `mod child;`
- parent/child.rs declares `mod grandchild;`
- parent/child/grandchild.rs is claimed by the root lib.rs crate

This tests that:
1. Adjacent file module discovery works (parent.rs instead of parent/mod.rs).
2. Deeply nested modules are correctly claimed through intermediate adjacent-style modules.
3. Unclaimed files in subdirectories still get their own rules.
