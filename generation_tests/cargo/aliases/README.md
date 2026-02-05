# Aliases Test

This test verifies that gazelle_rust correctly generates the `aliases` attribute a Cargo.toml uses
package renaming.

## Test Case

In `aliased_dep/Cargo.toml`:

```toml
[dependencies]
local_alias = { package = "external_crate", path = "../external_crate" }
```

This means the Rust code uses `local_alias::...` but the actual crate is `external_crate`.

## Expected Output

The generated BUILD.bazel should include:

```py
aliases = {"//external_crate:external_crate": "local_alias"}
```

This tells rules_rust to link `external_crate` but make it available in the code as `local_alias`.
