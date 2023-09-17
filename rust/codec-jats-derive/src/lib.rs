//! Provides a `JatsCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
#[darling(attributes(jats))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    elem: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(jats))]
struct FieldAttr {
    ident: Option<Ident>,
}

/// Derive the `JatsCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(JatsCodec, attributes(jats))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use proc_macro::TokenStream;

    let input = parse_macro_input!(input as DeriveInput);

    let attr = match TypeAttr::from_derive_input(&input) {
        Ok(value) => value,
        Err(error) => {
            return TokenStream::from(error.write_errors());
        }
    };

    let tokens = match &input.data {
        Data::Struct(..) => derive_struct(attr),
        Data::Enum(data) => derive_enum(attr, data),
        Data::Union(..) => return TokenStream::new(),
    };

    TokenStream::from(tokens)
}

/// Derive the `JatsCodec` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    quote! {
        impl JatsCodec for #struct_name {
            fn to_jats(&self) -> (String, Losses) {
                (String::new(), Losses::none())
            }
        }
    }
}

/// Derive the `JatsCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants_to_jats = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => {
                variants_to_jats.extend(quote! {
                    Self::#variant_name(v) => v.to_jats(),
                });
            }
            Fields::Unit => {
                variants_to_jats.extend(quote! {
                    Self::#variant_name => (stringify!(#variant_name).to_string(), Losses::none()),
                });
            }
        };
    }

    quote! {
        impl JatsCodec for #enum_name {
            fn to_jats(&self) -> (String, Losses) {
                match self {
                    #variants_to_jats
                }
            }
        }
    }
}
