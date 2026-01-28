load("@rules_shell//shell:sh_test.bzl", "sh_test")

def gofmt_test(name, srcs):
    filegroup_name = "{}/srcs".format(name)
    native.filegroup(
        name = filegroup_name,
        srcs = srcs,
        tags = ["manual"],
    )

    sh_test(
        name = name,
        srcs = ["//util:run_gofmt.sh"],
        data = ["@go_default_sdk//:bin/gofmt", filegroup_name],
        args = ["$(rootpaths {})".format(filegroup_name)],
        deps = ["@rules_shell//shell/runfiles"],
    )
