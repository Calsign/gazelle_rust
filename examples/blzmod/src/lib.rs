mod helper;

pub use helper::get_message;

#[cfg(test)]
mod tests {
    use crate::get_message;

    // test target will get created if there is at least one #[test] function
    #[test]
    fn foobar() {
        assert_eq!(get_message(), "Hello, gazelle_rust!")
    }
}
