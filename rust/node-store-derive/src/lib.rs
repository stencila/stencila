//! Provides `ReadNode` and `WriteNode` derive macros for structs and enums in Stencila Schema

mod read;
mod write;

#[proc_macro_derive(ReadNode)]
pub fn derive_read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    read::derive(input)
}

#[proc_macro_derive(WriteNode, attributes(key, autosurgeon))]
pub fn derive_write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    write::derive(input)
}
