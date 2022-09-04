fn foobar() {
    use a;
}

use x as y;

#[test]
fn foobar_test(arg: f::X) {
    use b;
    let z = y::something();

    // already used above, should not appear in test imports
    use a;
}

#[cfg(test)]
mod tests {
    use c;

    fn f1() {
        use d;
    }

    #[test]
    fn f2() {
        use e;
    }
}

#[cfg(feature = "foobar")]
mod other {
    use m;
}

#[cfg(x)]
mod other {
    use n;
}
