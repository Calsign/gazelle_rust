// #[gazelle::provides(...)] is handled correctly.

mod foo {
    mod bar1 {
        fn inner1() {}
    }

    mod bar2 {
        fn inner2() {}
    }
}

fn glob_usage() {
    mod x;

    #[gazelle::provides(bar1, bar2)]
    use foo::*;

    bar1::inner();
    bar2::inner();
}

#[gazelle::provides(fancy)]
fancy_macro!();

fn derived_usage() {
    let _x = fancy::FancyGenerated;
}
