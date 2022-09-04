use proc_macro::TokenStream;

#[proc_macro]
pub fn macro_msg(_item: TokenStream) -> TokenStream {
    let msg = lib::get_message();
    format!("fn macro_msg() -> &'static str {{ \"{}\" }}", msg)
        .parse()
        .unwrap()
}
