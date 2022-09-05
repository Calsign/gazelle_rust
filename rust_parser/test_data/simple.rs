extern crate test_extern_crate_1;

use test_use_1;
use test_use_2::Something;
use test_use_3::*;
use test_use_4 as outer_alias;
use test_use_5::self_use::{self, x};

use x::X;

use test_duplicate;

// TODO: derives aren't detected yet
#[derive(test_derive_1::Something)]
struct X {}

fn main() {
    println!("Hello, world!");

    test_inner_1::something();

    outer_alias::something();

    use test_duplicate;

    let x = X::new();

    self_use::something();

    use crate::foobar;
}

use test_inner_mod_3;

mod foobar {
    fn something() {}

    mod inner_mod_1 {}

    use inner_mod_1;

    mod test_inner_mod_2 {}

    mod test_inner_mod_3 {}
}

use foobar;

fn f(x: test_args_1::Something) -> test_ret_1::Something {
    use test_inner_2 as inner_alias;

    inner_alias::something();

    outer_alias::something();

    foobar::something();

    test_inner_mod_2::something();
}
