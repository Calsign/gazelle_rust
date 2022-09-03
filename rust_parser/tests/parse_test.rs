use std::env;
use std::error::Error;
use std::path::PathBuf;

struct TestCase {
    filename: &'static str,
    expected_imports: Vec<&'static str>,
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
                "test_duplicate",
                "test_inner_1",
                "test_args_1",
                "test_ret_1",
                "test_inner_2",
            ],
        },
    ];
}

#[test]
fn parse_test() -> Result<(), Box<dyn Error>> {
    let dir = if cfg!(feature = "bazel") {
        let mut d = PathBuf::from(env::var("RUNFILES_DIR")?);
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

        println!("{:?}", file);

        let mut imports = rust_parser::parse_imports(file)?;
        imports.sort();
        let mut expected = test_case.expected_imports.clone();
        expected.sort();

        assert_eq!(imports, expected);
    }

    Ok(())
}
