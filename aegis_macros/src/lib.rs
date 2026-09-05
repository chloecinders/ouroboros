use proc_macro::TokenStream;

mod command;
mod meta;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command::expand(attr, item)
}

#[proc_macro]
pub fn meta(item: TokenStream) -> TokenStream {
    meta::expand(item)
}
