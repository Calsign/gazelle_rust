module github.com/calsign/gazelle_rust

// After adding dependencies, run the following to update go_deps.bzl:
// bazel run //:gazelle_update_repos

go 1.22.4

require (
	github.com/bazelbuild/bazel-gazelle v0.37.0
	github.com/bazelbuild/rules_go v0.48.1
	google.golang.org/protobuf v1.34.2
)

require (
	github.com/bazelbuild/buildtools v0.0.0-20240313121412-66c605173954 // indirect
	golang.org/x/mod v0.16.0 // indirect
	golang.org/x/sys v0.18.0 // indirect
	golang.org/x/tools/go/vcs v0.1.0-deprecated // indirect
)
