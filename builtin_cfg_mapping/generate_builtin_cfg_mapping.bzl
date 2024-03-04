load("@bazel_skylib//lib:sets.bzl", "sets")
load("@bazel_skylib//rules:write_file.bzl", "write_file")
load(
    "@rules_rust//rust/platform:triple_mappings.bzl",
    "SUPPORTED_PLATFORM_TRIPLES",
    "abi_to_constraints",
    "cpu_arch_to_constraints",
    "system_to_constraints",
    "vendor_to_constraints",
)
load("@rules_rust//rust/platform:triple.bzl", "triple")

def configuration(flags):
    """
    Create a bazel configuration flag.
    """

    # TODO: support conjunctions/disjunctions of flags
    if len(flags) > 1:
        fail("Expected exactly one flag, got {}".format(flags))

    # Some rules_rust configurations are buggy and return "None" instead of
    # "none". This is a hack to fix them.
    return struct(flag = flags[0].replace("None", "none"))

def cfg(value, key = None):
    """
    Create a rust cfg flag.
    """

    return struct(
        key = key,
        value = value,
    )

def get_rules_rust_cfg_mappings():
    """
    Grab the triple mappings already defined in rules_rust. This covers most of
    the important cases.
    """

    cpu_arches = sets.make()
    systems = sets.make()
    vendors = sets.make()
    abis = sets.make()

    for trip_str in SUPPORTED_PLATFORM_TRIPLES:
        trip = triple(trip_str)
        sets.insert(cpu_arches, trip.arch)
        sets.insert(systems, trip.system)
        sets.insert(vendors, trip.vendor)
        if trip.abi:
            sets.insert(abis, trip.abi)

    mapping = {}

    for cpu_arch in sets.to_list(cpu_arches):
        constraints = cpu_arch_to_constraints(cpu_arch)
        if constraints:
            mapping[cfg(key = "target_arch", value = cpu_arch)] = configuration(constraints)

    for system in sets.to_list(systems):
        constraints = system_to_constraints(system)
        if constraints:
            mapping[cfg(key = "target_os", value = system)] = configuration(constraints)

    for vendor in sets.to_list(vendors):
        constraints = vendor_to_constraints(vendor)
        if constraints:
            mapping[cfg(key = "target_vendor", value = vendor)] = configuration(constraints)

    for abi in sets.to_list(abis):
        constraints = abi_to_constraints(abi)
        if constraints:
            # NOTE: Currently rust has no cfg flag for target_abi, so we do nothing here.
            # But there is an issue to create one: https://github.com/rust-lang/rust/issues/80970
            pass

    return mapping

def get_extra_cfg_mappings():
    return {
        cfg(key = "target_family", value = "unix"): configuration(["@rules_rust//rust/platform:unix"]),
        cfg("unix"): configuration(["@rules_rust//rust/platform:unix"]),
        cfg(key = "target_family", value = "windows"): configuration(["@platforms//os:windows"]),
        cfg("windows"): configuration(["@platforms//os:windows"]),
    }

def generate_builtin_cfg_mapping():
    """
    Generate a source file containing a mapping from well-known rust cfg flags.
    The goal is to cover all cfg flags that have well-known bazel configuration
    flags. This list serves as a reference:

    https://doc.rust-lang.org/reference/conditional-compilation.html
    """

    mapping = get_rules_rust_cfg_mappings() | get_extra_cfg_mappings()

    lines = []

    lines.append("package builtin_cfg_mapping")
    lines.append("var BuiltinCfgMapping = map[string]string {")

    for cfg, configuration in mapping.items():
        if cfg.key:
            cfg_str = "{}={}".format(cfg.key, cfg.value)
        else:
            cfg_str = cfg.value

        lines.append('  "{}": "{}",'.format(cfg_str, configuration.flag))

    lines.append("}")

    write_file(
        name = "builtin_cfg_mapping_src",
        out = "builting_cfg_mapping.go",
        content = lines,
        newline = "unix",
        visibility = ["//visibility:private"],
    )
