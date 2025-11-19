def gofmt_test(name, srcs):
    filegroup_name = "{}/srcs".format(name)
    native.filegroup(
        name = filegroup_name,
        srcs = srcs,
        tags = ["manual"],
    )

    native.sh_test(
        name = name,
        srcs = ["//util:run_gofmt.sh"],
        data = ["@go_default_sdk//:bin/gofmt", filegroup_name],
        args = ["$(rootpaths {})".format(filegroup_name)],
    )
