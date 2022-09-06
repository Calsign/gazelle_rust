load("@bazel_gazelle//:def.bzl", "gazelle", "gazelle_binary", "gazelle_generation_test")

def generation_tests():
    for file in native.glob(["**/WORKSPACE"]):
        # Name the test the path to the directory containing the WORKSPACE file.
        dir = file[0:-len("/WORKSPACE")]
        gazelle_generation_test(
            name = dir,
            gazelle_binary = "//:gazelle_bin",
            test_data = native.glob([dir + "/**"]),
            # TODO: It seems like the parser <--> language plugin streams get crossed if the tests
            # run in parallel. Figure out why.
            tags = ["exclusive"],
        )
