//! Provides a `Strip` derive macro for structs and enums in Stencila Schema

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields},
};

/// Derive the `Strip` trait for a `struct` or `enum`
#[proc_macro_derive(Strip)]
pub fn derive_strip(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `Strip` trait for a `struct`
fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_ident = &field.ident;
        let field_name = &field_ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();

        let field = if field_name == "id" {
            quote! {
                if targets.id {
                    self.id = None;
                }
            }
        } else if field_name != "r#type" {
            quote! {
                self.#field_ident.strip(targets);
            }
        } else {
            continue;
        };

        fields.extend(field);
    }

    quote! {
        impl node_strip::Strip for #struct_name {
            fn strip(&mut self, targets: &node_strip::Targets) -> &mut Self {
                #fields
                self
            }
        }
    }
}

/// Derive the `Strip` trait for an `enum`
fn derive_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => { v.strip(targets); },
            },
            Fields::Unit => continue,
        };
        cases.extend(case)
    }

    if cases.is_empty() {
        quote! {
            impl node_strip::Strip for #enum_name {}
        }
    } else {
        quote! {
            impl node_strip::Strip for #enum_name {
                fn strip(&mut self, targets: &node_strip::Targets) -> &mut Self {
                    match self {
                        #cases
                    }
                    self
                }
            }
        }
    }
}
