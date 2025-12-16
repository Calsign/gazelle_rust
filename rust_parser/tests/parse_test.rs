use runfiles::Runfiles;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::path::PathBuf;

struct TestCase {
    filename: &'static str,
    enabled_features: Vec<&'static str>,
    expected_imports: Vec<&'static str>,
    expected_test_imports: Vec<&'static str>,
    expected_extern_mods: Vec<&'static str>,
}

lazy_static::lazy_static! {
    static ref TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            filename: "simple.rs",
            enabled_features: vec![],
            expected_imports: vec![
                "gazelle",
                "test_extern_crate_1",
                "test_use_1",
                "test_use_2",
                "test_use_3",
                "test_use_4",
                "test_use_5",
                "test_duplicate",
                "test_inner_1",
                "test_args_1",
                "test_ret_1",
                "test_inner_2",
                "test_inner_mod_2",
                "test_inner_mod_3",
                "test_derive_1",
                "test_attribute_1",
                "test_same_name",
                "test_cfg_attr_derive",
                "test_cfg_attr_macro",
                "test_cfg_attr_macro_on_impl",
                "test_cfg_attr_macro_on_fn",
                "test_bare_use_group1",
                "test_bare_use_group2",
            ],
            expected_test_imports: vec![],
            expected_extern_mods: vec!["extern_mod"],
        },
        TestCase {
            filename: "test_only.rs",
            enabled_features: vec![],
            expected_imports: vec![
                "a",
                "x",
            ],
            expected_test_imports: vec![
                "b",
                "c",
                "d",
                "e",
                "f",
            ],
            expected_extern_mods: vec![],
        },
        TestCase {
            filename: "early_mod.rs",
            enabled_features: vec![],
            expected_imports: vec!["ee"],
            expected_test_imports: vec![],
            expected_extern_mods: vec![],
        },
        TestCase {
            filename: "features.rs",
            enabled_features: vec!["bar"],
            expected_imports: vec![
                "bar",
                "baz",
                "qux",
                "test_extern_crate_2",
                "test_extern_crate_3",
            ],
            expected_test_imports: vec![],
            expected_extern_mods: vec![
                "extern_mod_2",
            ],
        },
    ];
}

fn assert_eq_vecs(actual: &[String], expected: &[String], msg: &str) {
    let actual_set: HashSet<_> = actual.iter().collect();
    let expected_set: HashSet<_> = expected.iter().collect();
    if actual_set != expected_set {
        let mut only_actual: Vec<_> = actual_set.difference(&expected_set).collect();
        only_actual.sort();
        let mut only_expected: Vec<_> = expected_set.difference(&actual_set).collect();
        only_expected.sort();

        if !only_actual.is_empty() {
            println!("Only in actual:");
            for item in only_actual {
                println!("  {}", item);
            }
        }
        if !only_expected.is_empty() {
            println!("Only in expected:");
            for item in only_expected {
                println!("  {}", item);
            }
        }
        panic!("vecs differ: {msg}");
    }
}

#[test]
fn parse_test() -> Result<(), Box<dyn Error>> {
    let dir = if cfg!(feature = "bazel") {
        let r = Runfiles::create().unwrap();
        r.rlocation("_main/rust_parser/test_data/").unwrap()
    } else {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data");
        d
    };

    for test_case in &*TEST_CASES {
        let mut file = dir.clone();
        file.push(test_case.filename);
        let enabled_features: Vec<String> = test_case
            .enabled_features
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rust_imports = parser::parse_imports(file, &enabled_features)?;
        assert_eq_vecs(
            &rust_imports.imports,
            &test_case
                .expected_imports
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "imports",
        );
        assert_eq_vecs(
            &rust_imports.test_imports,
            &test_case
                .expected_test_imports
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "test_imports",
        );
        assert_eq_vecs(
            &rust_imports.extern_mods,
            &test_case
                .expected_extern_mods
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "extern_modes",
        );
    }

    Ok(())
}
