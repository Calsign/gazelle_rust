use proc_macro::TokenStream;

/// This is a dummy macro that returns the token stream unchanged. It is parsed by gazelle to
/// understand that the attached use item should be ignored, and no bazel dependency should be
/// added to the corresponding target.
#[proc_macro_attribute]
pub fn ignore(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
