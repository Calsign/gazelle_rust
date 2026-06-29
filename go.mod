module github.com/calsign/gazelle_rust

// After adding dependencies, run the following to update go_deps.bzl:
// bazel run //:gazelle_update_repos

go 1.26.4

require (
	github.com/bazelbuild/bazel-gazelle v0.51.3
	github.com/bazelbuild/rules_go v0.61.1
	google.golang.org/protobuf v1.36.10
)

require (
	github.com/bazel-contrib/bazel-gazelle/v2 v2.0.0-2 // indirect
	github.com/bazelbuild/buildtools v0.0.0-20250930140053-2eb4fccefb52 // indirect
	golang.org/x/mod v0.25.0 // indirect
	golang.org/x/sys v0.33.0 // indirect
	golang.org/x/tools/go/vcs v0.1.0-deprecated // indirect
)
