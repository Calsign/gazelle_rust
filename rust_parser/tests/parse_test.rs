use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::path::PathBuf;

struct TestCase {
    filename: &'static str,
    expected_imports: Vec<&'static str>,
    expected_test_imports: Vec<&'static str>,
}

lazy_static::lazy_static! {
    static ref TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            filename: "simple.rs",
            expected_imports: vec![
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
            ],
            expected_test_imports: vec![],
        },
        TestCase {
            filename: "test_only.rs",
            expected_imports: vec![
                "a",
                "x",
                "m",
                "n",
            ],
            expected_test_imports: vec![
                "b",
                "c",
                "d",
                "e",
                "f",
            ],
        },
    ];
}

fn assert_eq_vecs(actual: &[String], expected: &[String]) {
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
        panic!("vecs differ");
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
        let mut file = dir.clone();
        file.push(test_case.filename);

        let rust_imports = parser::parse_imports(file)?;
        assert_eq_vecs(
            &rust_imports.imports,
            &test_case
                .expected_imports
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
        assert_eq_vecs(
            &rust_imports.test_imports,
            &test_case
                .expected_test_imports
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
    }

    Ok(())
}
