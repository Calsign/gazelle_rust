fn foo1 () {
    println!(foo1::my_fn())
}

fn bar1 () -> &'static str {
    println!(bar1::my_fn());
    "bar"
}

fn foo2 () {
    println!("Here goes: {}", foo2::my_fn())
}

fn bar2 () -> &'static str {
    println!("Here goes: {}", bar2::my_fn());
    "bar"
}

fn baz1 () {
    println!(include_str!("file1.txt"))
}

fn baz2 () -> &'static str {
    println!(include_str!("file2.txt"));
    "baz"
}

fn baz3 () {
    println!("Here goes: {}", include_str!("file3.txt"))
}

fn baz4 () -> &'static str {
    println!("Here goes: {}", include_str!("file4.txt"));
    "baz"
}

fn nested() {
    assert_eq!(
        tera.render(
            "t",
            &nested1::Context::from_value(nested2::json!({
                "contextual_data": {
                    "foo": "bar",
                },
            }))
            .unwrap()
        )
        .inspect_err(|e| {
            println!("{e}");
        })
        .unwrap(),
        r#"
            1
            2
            3
        "#
    );
}

fn leading_ident_to_skip() {
    assert_eq!(rendered_prompt, include_str!("file5.txt"));
}

fn fancy() {
    assert_eq!(
        strip_html(include_str!("file6.txt"), false),
        include_str!("file7.txt"),
    );
}
