# go/gazelle
load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")
load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")

# protobuf
load("@rules_proto//proto:repositories.bzl", "rules_proto_dependencies", "rules_proto_toolchains")
load("@rules_rust//proto/protobuf:repositories.bzl", "rust_proto_protobuf_dependencies", "rust_proto_protobuf_register_toolchains")

# versions of dependencies
load(":deps_versions.bzl", "versions")

def gazelle_rust_dependencies2():
    # go/gazelle
    go_rules_dependencies()

    if "go_sdk" not in native.existing_rules():
        go_register_toolchains(version = versions.GO_VERSION)

    gazelle_dependencies()

    # protobuf
    rules_proto_dependencies()
    rules_proto_toolchains()

    rust_proto_protobuf_dependencies()
    rust_proto_protobuf_register_toolchains()
