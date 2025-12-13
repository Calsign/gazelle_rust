// Conditional `use` imports.

#[cfg(feature = "foo")]
use foo;

#[cfg(feature = "bar")]
use bar;

#[cfg(feature = "baz", "bar")]
use baz;

#[cfg(all(feature = "bar", not(feature = "foo")))]
use qux;

#[cfg(all(feature = "baz", not(feature = "foo")))]
use quux;

// Conditional `extern crate`

#[cfg(feature = "foo")]
extern crate test_extern_crate_1;

#[cfg(feature = "bar")]
extern crate test_extern_crate_2;

#[cfg(feature = "baz", "bar")]
extern crate test_extern_crate_3;

// Conditional `extern mod`

#[cfg(feature = "foo")]
mod extern_mod_1;

#[cfg(feature = "foo")]
use extern_mod_1::SomeExternThing;

#[cfg(feature = "bar")]
mod extern_mod_2;

#[cfg(feature = "bar")]
use extern_mod_2::SomeExternThing;
