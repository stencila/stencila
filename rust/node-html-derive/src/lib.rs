//! Provides `ToHtml` derive macros for structs and enums in Stencila Schema

mod to_html;

#[proc_macro_derive(ToHtml)]
pub fn derive_to_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    to_html::derive(input)
}
