#[cfg(test)]
fn foobar1() {
    use dep1::A;
}

#[cfg(unix)]
fn foobar2() {
    use dep2::A;
}

#[cfg(target_family = "unix")]
fn foobar3() {
    use dep3::A;
}

#[cfg(all(test, target_family = "windows"))]
fn foobar4() {
    use dep4::A;
}

// TODO(will): support cfg_attr
#[cfg_attr(test, derive(dep_cfg_attr::A))]
struct Foobar {
    b: dep5::A,
}

#[cfg(unix)]
mod mod1 {
    #[cfg(test)]
    fn foobar5() {
        use dep6::A;
    }
}

#[cfg(test)]
mod foobar {
    use dep7::A;
}

#[cfg(feature = "some_feature")]
fn gated_behind_feature() {
    use dep8::A;
}

#[cfg(a)]
mod x1 {
    use two_paths::A;
}

#[cfg(b)]
mod x2 {
    use two_paths::B;
}
