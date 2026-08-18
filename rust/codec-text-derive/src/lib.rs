//! Provides the `TextCodec` derive macro for structs and enums in Stencila Schema

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, parse_macro_input};

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
    // being included in text. Use only one field for a struct, chosen from two tiers.

    // The main content of a type, which is what its text is whenever it has any
    const CONTENT: &[&str] = &[
        "content", // Content of most block and inline types
        "items",   // List content
        "rows",    // Table content
        "cells",   // TableRow content
        "code",    // Code and math content
    ];

    // Failing that, the property carrying the type's identity, so that a type whose
    // whole meaning is in a single value or name -- a `Date`, an `Organization`, a
    // `PropertyValue` -- reads as something rather than as an empty string.
    const IDENTITY: &[&str] = &[
        "value", // Date, DateTime, Time, Timestamp, Duration, PropertyValue, ...
        "name",  // Organization, Periodical, Variable, Parameter, ...
    ];

    // The tiers are consulted in order, rather than being one list, because several
    // types declare a `name` before their content -- `Agent`, `File`, `Prompt`, `Skill`
    // and `Workflow` among them -- and the text of those is their content, not their
    // name. Within a tier the first matching field in declaration order wins.
    let field_named_in = |names: &[&str]| {
        data.fields.iter().find_map(|field| {
            let field_ident = field.ident.as_ref()?;
            names
                .contains(&field_ident.to_string().as_str())
                .then_some(field_ident)
        })
    };

    let mut fields = TokenStream::new();
    if let Some(field_ident) = field_named_in(CONTENT).or_else(|| field_named_in(IDENTITY)) {
        fields.extend(quote! {
            let mut text = self.#field_ident.to_text();
        });
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
