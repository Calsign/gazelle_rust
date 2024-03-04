use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::path::PathBuf;

use parser::{BExprAtom, ConfigFlag};

struct TestCase {
    filename: &'static str,
    expected_imports: Vec<(&'static str, ConfigFlag)>,
    expected_extern_mods: Vec<&'static str>,
}

lazy_static::lazy_static! {
    static ref TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            filename: "simple.rs",
            expected_imports: vec![
                ("gazelle", ConfigFlag::Const(true)),
                ("test_extern_crate_1", ConfigFlag::Const(true)),
                ("test_use_1", ConfigFlag::Const(true)),
                ("test_use_2", ConfigFlag::Const(true)),
                ("test_use_3", ConfigFlag::Const(true)),
                ("test_use_4", ConfigFlag::Const(true)),
                ("test_use_5", ConfigFlag::Const(true)),
                ("test_duplicate", ConfigFlag::Const(true)),
                ("test_inner_1", ConfigFlag::Const(true)),
                ("test_args_1", ConfigFlag::Const(true)),
                ("test_ret_1", ConfigFlag::Const(true)),
                ("test_inner_2", ConfigFlag::Const(true)),
                ("test_inner_mod_2", ConfigFlag::Const(true)),
                ("test_inner_mod_3", ConfigFlag::Const(true)),
                ("test_derive_1", ConfigFlag::Const(true)),
                ("test_attribute_1", ConfigFlag::Const(true)),
            ],
            expected_extern_mods: vec!["extern_mod"],
        },
        TestCase {
            filename: "test_only.rs",
            expected_imports: vec![
                ("a", ConfigFlag::Const(true)),
                ("x", ConfigFlag::Const(true)),
                ("m", ConfigFlag::Terminal(BExprAtom::KeyOption {
                    key: "feature".to_string(),
                    value: "foobar".to_string(),
                })),
                ("n", ConfigFlag::Terminal(BExprAtom::Option { option: "x".to_string() })),
                ("b", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("c", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("d", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("e", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("f", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
            ],
            expected_extern_mods: vec![],
        },
        TestCase {
            filename: "cfg.rs",
            expected_imports: vec![
                ("dep1", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("dep2", ConfigFlag::Terminal(BExprAtom::Option { option: "unix".to_string() })),
                ("dep3", ConfigFlag::Terminal(BExprAtom::KeyOption {
                    key: "target_family".to_string(),
                    value: "unix".to_string(),
                })),
                ("dep4", ConfigFlag::And(
                    Box::new(ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                    Box::new(ConfigFlag::Terminal(BExprAtom::KeyOption {
                        key: "target_family".to_string(),
                        value: "windows".to_string(),
                    })),
                )),
                ("dep5", ConfigFlag::Const(true)),
                ("dep6", ConfigFlag::And(
                    Box::new(ConfigFlag::Terminal(BExprAtom::Option { option: "unix".to_string() })),
                    Box::new(ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                )),
                ("dep7", ConfigFlag::Terminal(BExprAtom::Option { option: "test".to_string() })),
                ("dep8", ConfigFlag::Terminal(BExprAtom::KeyOption {
                    key: "feature".to_string(),
                    value: "some_feature".to_string(),
                })),
                ("two_paths", ConfigFlag::Or(
                    Box::new(ConfigFlag::Terminal(BExprAtom::Option { option: "b".to_string() })),
                    Box::new(ConfigFlag::Terminal(BExprAtom::Option { option: "a".to_string() })),
                )),
            ],
            expected_extern_mods: vec![],
        },
    ];
}

fn assert_eq_vecs<T>(actual: &[T], expected: &[T])
where
    T: PartialEq + std::fmt::Debug + Eq + std::hash::Hash + std::cmp::Ord,
{
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
                println!("  {:?}", item);
            }
        }
        if !only_expected.is_empty() {
            println!("Only in expected:");
            for item in only_expected {
                println!("  {:?}", item);
            }
        }
        panic!("vecs differ");
    }
}

fn assert_eq_maps<K, V>(actual: &HashMap<K, V>, expected: &HashMap<K, V>)
where
    K: PartialEq + std::fmt::Debug + Eq + std::hash::Hash + std::cmp::Ord,
    V: PartialEq + std::fmt::Debug,
{
    if actual != expected {
        let actual_keys = actual.keys().collect::<HashSet<_>>();
        let expected_keys = expected.keys().collect::<HashSet<_>>();

        let mut only_actual: Vec<_> = actual_keys.difference(&expected_keys).collect();
        only_actual.sort();
        let mut only_expected: Vec<_> = expected_keys.difference(&actual_keys).collect();
        only_expected.sort();

        if !only_actual.is_empty() {
            println!("Only in actual:");
            for key in only_actual {
                println!("  {:?} => {:?}", key, actual[key]);
            }
        }

        if !only_expected.is_empty() {
            println!("Only in expected:");
            for key in only_expected {
                println!("  {:?} => {:?}", key, expected[key]);
            }
        }

        for (key, actual_value) in actual.iter() {
            if let Some(expected_value) = expected.get(key) {
                if actual_value != expected_value {
                    println!(
                        "Mismatched values for key {:?}: expected {:?}, got {:?}",
                        key, expected_value, actual_value
                    );
                }
            }
        }

        panic!("maps differ");
    }
}

#[test]
fn parse_test() -> Result<(), Box<dyn Error>> {
    let dir = if cfg!(feature = "bazel") {
        let mut d = runfiles::find_runfiles_dir()?;
        d.push("gazelle_rust/rust_parser/test_data");
        d
    } else {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data");
        d
    };

    for test_case in &*TEST_CASES {
        eprintln!("Testing {}", test_case.filename);

        let mut file = dir.clone();
        file.push(test_case.filename);

        let rust_imports = parser::parse_imports(file)?;
        assert_eq_maps(
            &rust_imports.imports,
            &test_case
                .expected_imports
                .iter()
                .map(|(s, cfg)| (s.to_string(), cfg.clone()))
                .collect::<HashMap<_, _>>(),
        );
        assert_eq_vecs(
            &rust_imports.extern_mods,
            &test_case
                .expected_extern_mods
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
    }

    Ok(())
}
