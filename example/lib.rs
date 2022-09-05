mod helper;

pub use helper::get_message;

#[cfg(test)]
mod tests {
    // demonstrate a test-only dependency
    use crate::get_message;
    use macro_lib::macro_msg;

    macro_msg!();

    // test target will get created if there is at least one #[test] function
    #[test]
    fn foobar() {
        assert_eq!(macro_msg(), get_message())
    }
}
