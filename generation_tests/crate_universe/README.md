
Tests that crate_universe dependencies are handled correctly. Each subdirectory is a different test.

To repin the lockfiles (or vendored targets) for one of these tests, run:

```bash
bazel run //generation_tests/crate_universe:repin -- <test directory name>
```

Or run with no argument to repin all.

Warning: Use the update script at your own risk! It probably only works on Linux, and it is not
tested for things like paths containing spaces.
