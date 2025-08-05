//! Provides the `TextCodec` derive macro for structs and enums in Stencila Schema

use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, parse_macro_input};

use common::{proc_macro2::TokenStream, quote::quote};

/// Derive the `TextCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(TextCodec)]
pub fn derive_to_text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    let struct_ident = &input.ident;

    let struct_name = struct_ident.to_string();
    let struct_name = struct_name.as_str();

    if struct_name == "Text" {
        // Instead of having attributes for skipping / having special
        // function (as with other codecs), just use this one-off if clause
        return quote! {
            impl TextCodec for Text {
                fn to_text(&self) -> String {
                    self.value.to_string()
                }
            }
        };
    }

    // Only treat certain properties as having text content. This avoid string
    // properties like `programmingLanguage` and enums like `List.order` from
    // being included in text. Use only the one, main "content" field for a struct.
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let field_name = field_ident.to_string();
        let field_name = field_name.as_str();

        if matches!(
            field_name,
            | "content" // Content of most block and inline types
                | "items" // List content
                | "rows" // Table content
                | "cells" // TableRow content
                | "code" // Code and math content
        ) {
            fields.extend(quote! {
                let mut text = self.#field_ident.to_text();
            });
            break;
        }
    }

    // Modify end for certain node types to give some whitespace structuring to
    // the otherwise plain text content
    let end = if matches!(
        struct_name,
        "CodeBlock"
            | "CodeChunk"
            | "Figure"
            | "Heading"
            | "MathBlock"
            | "Paragraph"
            | "RawBlock"
            | "Table"
    ) {
        quote! {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            if !text.ends_with("\n\n") {
                text.push('\n');
            }
        }
    } else if matches!(struct_name, "TableRow") {
        quote! {
            while text.ends_with(' ') {
                text.pop();
            }
            if !text.ends_with('\n') {
                text.push('\n');
            }
        }
    } else if matches!(struct_name, "TableCell") {
        quote! {
            while text.ends_with('\n') {
                text.pop();
            }
            text.push(' ');
        }
    } else {
        quote! {}
    };

    quote! {
        impl TextCodec for #struct_ident {
            fn to_text(&self) -> String {
                let mut text = String::new();

                #fields
                #end

                text
            }
        }
    }
}

/// Derive the `TextCodec` trait for an `enum`
fn derive_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_ident = &input.ident;

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.to_text(),
            },
            Fields::Unit => quote! {
                Self::#variant_name => stringify!(#variant_name).to_string(),
            },
        };
        variants.extend(case)
    }

    quote! {
        impl TextCodec for #enum_ident {
            fn to_text(&self) -> String {
                match self {
                    #variants
                }
            }
        }
    }
}
