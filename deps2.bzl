# go/gazelle
load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")
load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")
load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

# protobuf
load("@rules_proto//proto:repositories.bzl", "rules_proto_dependencies", "rules_proto_toolchains")
load("@rules_rust//proto:repositories.bzl", "rust_proto_repositories")

def gazelle_rust_dependencies2():
    # go/gazelle
    go_rules_dependencies()

    if "go_sdk" not in native.existing_rules():
        go_register_toolchains(version = "1.18.3")

    gazelle_dependencies()

    # protobuf
    rules_proto_dependencies()
    rules_proto_toolchains()

    rust_proto_repositories()
