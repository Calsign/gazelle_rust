load("@gazelle//:def.bzl", "gazelle_generation_test")

def generation_tests(disabled_tests):
    disabled_tests = {test_name: False for test_name in disabled_tests}

    for file in native.glob(["**/WORKSPACE"]):
        # Name the test the path to the directory containing the WORKSPACE file.
        dir = file[0:-len("/WORKSPACE")]

        tags = []

        if dir in disabled_tests:
            tags.append("manual")
            disabled_tests[dir] = True

        gazelle_generation_test(
            name = dir,
            gazelle_binary = "//:gazelle_bin",
            test_data = native.glob([dir + "/**"]),
            tags = tags,
        )

    for test_name, found in disabled_tests.items():
        if found == False:
            fail("Test {} is disabled, but wasn't found".format(test_name))
