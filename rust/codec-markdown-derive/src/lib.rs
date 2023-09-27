//! Provides the `MarkdownCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
#[darling(attributes(markdown))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    format: Option<String>,

    #[darling(default)]
    escape: Option<String>,

    #[darling(default)]
    special: bool,
}

#[derive(FromField)]
#[darling(attributes(markdown))]
struct FieldAttr {
    ident: Option<Ident>,
}

/// Derive the `MarkdownCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(MarkdownCodec, attributes(markdown))]
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

/// Derive the `MarkdownCodec` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    if type_attr.special {
        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self) -> (String, Losses) {
                    self.to_markdown_special()
                }
            }
        }
    } else if let Some(format) = type_attr.format {
        // When a format is provided record loss of properties not interpolated in it

        let mut fields = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return
            };

            if field_name == "r#type" {
                // Skip the type field
                return;
            }

            let field_tokens = if format.contains(&["{", &field_name.to_string(), "}"].concat()) {
                let mut tokens = quote! {
                    let (#field_name, field_losses) = self.#field_name.to_markdown();
                    losses.merge(field_losses);
                };
                if let Some(escape) = &type_attr.escape {
                    tokens.extend(quote! {
                        let #field_name = #field_name.replace(#escape, &[r"\", #escape].concat());
                    });
                }
                tokens
            } else {
                quote! {
                    losses.add(stringify!(#field_name));
                }
            };
            fields.extend(field_tokens);
        });

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self) -> (String, Losses) {
                    let mut losses = Losses::none();

                    #fields

                    (format!(#format), losses)
                }
            }
        }
    } else {
        // Fallback is to encode all fields but to record loss of structure

        let mut fields = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return
            };

            if field_name == "r#type" {
                // Skip the type field
                return;
            }

            let field_tokens = {
                quote! {
                    let (field_markdown, field_losses) = self.#field_name.to_markdown();
                    markdown.push_str(&field_markdown);
                    losses.merge(field_losses);
                }
            };
            fields.extend(field_tokens)
        });

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self) -> (String, Losses) {
                    let mut markdown = String::new();
                    let mut losses = Losses::one(concat!(stringify!(#struct_name), "#"));

                    #fields

                    (markdown, losses)
                }
            }
        }
    }
}

/// Derive the `MarkdownCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_tokens = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.to_markdown(),
            },
            Fields::Unit => quote! {
                Self::#variant_name => (stringify!(#variant_name).to_string(), Losses::none()),
            },
        };
        variants.extend(variant_tokens)
    }

    quote! {
        impl MarkdownCodec for #enum_name {
            fn to_markdown(&self) -> (String, Losses) {
                match self {
                    #variants
                }
            }
        }
    }
}
