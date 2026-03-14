// Use statements are allowed to appear after the code that uses them.

use x::X;

mod x {
    pub struct X;
}

mod a {
    pub mod aa {
        pub struct A;
    }
}

fn fn1() {
    let a = aa::A;

    use a::aa;
}

mod b {
    pub mod bb {
        pub struct B;
    }
}

mod y {
    type Y = bb::B;

    use super::b::bb;
}

mod c {
    pub mod cc {
        pub struct C;
    }
}

mod d {
    pub mod dd {
        pub struct D;
    }
}

fn fn2() {
    if true {
        let c = cc::C;
        // this one is still in scope
        let d = dd::D;

        use c::cc;
    }

    use d::dd;
}

mod e {
    pub mod ee {
        pub struct E;
    }
}

mod f {
    use super::e::ee;
}

// this one shows up as a dependency because use statements are only in scope for the block where
// they appear
type G = ee::E;
