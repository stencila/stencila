//! Provides the `TextCodec` derive macro for structs and enums in Stencila Schema

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields},
};

/// Derive the `TextCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(TextCodec)]
pub fn derive_to_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `TextCodec` trait for a `struct`
fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    if struct_name == "Text" {
        // Instead of having attributes for skipping / having special
        // function (as with other codecs), just use this one-off if clause
        return quote! {
            impl TextCodec for Text {
                fn to_text(&self) -> (String, Losses) {
                    (self.value.to_string(), Losses::none())
                }
            }
        };
    }

    // Do not record loss of structure for options structs
    let losses = if struct_name.to_string().ends_with("Options") {
        quote!(Losses::none())
    } else {
        quote!(Losses::one(concat!(stringify!(#struct_name), "#")))
    };

    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_indent = &field.ident;
        let field_name = &field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();

        if field_name == "r#type"
            || field_name == "uid"
            || field_name == "authors"
            || field_name == "provenance"
        {
            continue;
        }

        let field = {
            quote! {
                let (field_text, field_losses) = self.#field_indent.to_text();
                text.push_str(&field_text);
                losses.merge(field_losses);
            }
        };
        fields.extend(field);
    }

    quote! {
        impl TextCodec for #struct_name {
            fn to_text(&self) -> (String, Losses) {
                let mut text = String::new();
                let mut losses = #losses;

                #fields

                (text, losses)
            }
        }
    }
}

/// Derive the `TextCodec` trait for an `enum`
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
        impl TextCodec for #enum_name {
            fn to_text(&self) -> (String, Losses) {
                match self {
                    #cases
                }
            }
        }
    }
}
