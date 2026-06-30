
# contributing to gazelle\_rust

Thanks for contributing!

## Project structure

 * `rust_language`: golang gazelle language plugin, this is the main entrypoint
 * `rust_parser`: rust binary invoked by the language plugin to parse imports
 * `proto`: protobuf API for communication between `rust_language` and `rust_parser`
 * `gazelle_rust_parser`: core library for parsing rust imports used by `rust_parser`
 * `macro`: implementation of `#[gazelle::foo]` directives as proc macros

For more information about writing gazelle language plugins, see gazelle's docs on
[extending gazelle](https://github.com/bazel-contrib/bazel-gazelle/blob/master/extend.md).

## Running CI locally

Make sure you're using bazelisk so that you get the correct bazel version.

Run CI with:
```bash
./ci.sh
```

Note that this is not just `bazel test //...`, but also building the `./example` directory and
verifying "gazelle invariance" (i.e. ensuring that running gazelle results in no changes) in both
gazelle\_rust and `./example`.

## AI policy

Use of AI tools in contributions is allowed but must be disclosed.
