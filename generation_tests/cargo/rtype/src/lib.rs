mod foobar;
mod r#type;

fn hello() -> &'static str {
    foo();
    bar();
    "hello"
}
