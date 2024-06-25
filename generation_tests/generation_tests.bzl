load("@gazelle//:def.bzl", "gazelle_generation_test")

def generation_tests():
    for file in native.glob(["**/WORKSPACE"]):
        # Name the test the path to the directory containing the WORKSPACE file.
        dir = file[0:-len("/WORKSPACE")]
        gazelle_generation_test(
            name = dir,
            gazelle_binary = "//:gazelle_bin",
            test_data = native.glob([dir + "/**"]),
        )
