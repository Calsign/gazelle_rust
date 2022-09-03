extern crate test_extern_crate_1;

use test_use_1;
use test_use_2::Something;
use test_use_3::*;
use test_use_4 as outer_alias;

use test_duplicate;

// TODO: derives aren't detected yet
#[derive(derive_1::Something)]
fn main() {
    println!("Hello, world!");

    test_inner_1::something();

    outer_alias::something();

    use test_duplicate;
}

fn f(x: test_args_1::Something) -> test_ret_1::Something {
    use test_inner_2 as inner_alias;

    inner_alias::something();

    outer_alias::something();
}
