//! Provides `ToText` derive macro for structs and enums in Stencila Schema

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields},
};

/// Derive the `ToText` trait for a `struct` or an `enum`
#[proc_macro_derive(ToText)]
pub fn derive_to_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `ToText` trait for a `struct`
fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_name = &field.ident;
        let field_name_string = &field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        if field_name_string == "r#type" {
            continue;
        }

        let field = {
            quote! {
                let (field_text, mut field_losses) = self.#field_name.to_text();
                text.push_str(&field_text);
                losses.append(&mut field_losses);
            }
        };
        fields.extend(field);
    }

    quote! {
        impl codec_text_traits::ToText for #struct_name {
            fn to_text(&self) -> (String, Losses) {
                let mut text = String::new();
                let mut losses = Losses::new([Loss::of_structure(
                    LossDirection::Encode,
                    stringify!(#struct_name)
                )]);

                #fields

                (text, losses)
            }
        }
    }
}

/// Derive the `ToText` trait for an `enum`
fn derive_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.to_text(),
            },
            Fields::Unit => quote! {
                Self::#variant_name => (stringify!(#variant_name).to_string(), Losses::none()),
            },
        };
        cases.extend(case)
    }

    quote! {
        impl codec_text_traits::ToText for #enum_name {
            fn to_text(&self) -> (String, Losses) {
                match self {
                    #cases
                }
            }
        }
    }
}
