mod foobar;

fn hello() -> &'static str {
    include_str!("file1.txt")
}
