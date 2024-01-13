//! Provides the `MarkdownCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident, PathSegment, Type},
};

#[derive(FromDeriveInput)]
#[darling(attributes(markdown))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    template: Option<String>,

    #[darling(default)]
    escape: Option<String>,

    #[darling(default)]
    special: bool,
}

#[derive(FromField)]
#[darling(attributes(markdown))]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    flatten: bool,
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
                fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
                    self.to_markdown_special(context)
                }
            }
        }
    } else if let Some(template) = type_attr.template {
        // When a format is provided record loss of properties not interpolated in it

        let mut fields = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return
            };

            if field_name == "r#type" || field_name == "uid" {
                return;
            }

            let field_tokens = if template.contains(&["{", &field_name.to_string(), "}"].concat()) {
                let mut tokens = quote! {
                    let (#field_name, field_losses) = self.#field_name.to_markdown(context);
                    losses.merge(field_losses);
                };
                if let Some(escape) = &type_attr.escape {
                    tokens.extend(quote! {
                        let #field_name = #field_name.replace(#escape, &[r"\", #escape].concat());
                    });
                }
                tokens
            } else if field_attr.flatten {
                // The best that we can do here is to add an loss for options,
                // if any options are `Some`, as we can not easily get higher granularity
                quote! {
                    let (field_md, _) = self.#field_name.to_markdown(context);
                    if !field_md.is_empty() {
                        losses.add(concat!(stringify!(#struct_name), ".options"));
                    }
                }
            } else {
                let Type::Path(type_path) = field_attr.ty else {
                    return
                };
                let Some(PathSegment{ident: field_type,..}) = type_path.path.segments.last() else {
                    return
                };

                let record_loss = quote! {
                    losses.add(concat!(stringify!(#struct_name), ".", stringify!(#field_name)));
                };

                if field_type == "Option" {
                    quote! { if self.#field_name.is_some() { #record_loss }}
                } else if field_type == "Vec" {
                    quote! { if !self.#field_name.is_empty() { #record_loss }}
                } else {
                    record_loss
                }
            };
            fields.extend(field_tokens);
        });

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
                    let mut losses = Losses::none();

                    #fields

                    (format!(#template), losses)
                }
            }
        }
    } else {
        // Fallback is to encode all fields but to record loss of structure of this type
        // (but not for XxxxOptions)

        let mut fields = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return
            };

            if field_name == "r#type" || field_name == "uid" {
                return;
            }

            let field_tokens = quote! {
                let (field_markdown, field_losses) = self.#field_name.to_markdown(context);
                markdown.push_str(&field_markdown);
                losses.merge(field_losses);
            };
            fields.extend(field_tokens)
        });

        let losses = if struct_name.to_string().ends_with("Options") {
            quote!(Losses::none())
        } else {
            quote!(Losses::one(concat!(stringify!(#struct_name), "#")))
        };

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
                    let mut markdown = String::new();
                    let mut losses = #losses;

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
                Self::#variant_name(v) => v.to_markdown(context),
            },
            Fields::Unit => quote! {
                Self::#variant_name => (stringify!(#variant_name).to_string(), Losses::none()),
            },
        };
        variants.extend(variant_tokens)
    }

    quote! {
        impl MarkdownCodec for #enum_name {
            fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
                match self {
                    #variants
                }
            }
        }
    }
}
