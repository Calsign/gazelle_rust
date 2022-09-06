use non_test_lib::*;

#[test]
fn this_is_a_test() {
    use test_lib::*;
}

#[cfg(test)]
mod tests {
    use test_lib::*;
}
