//! Provides the `MarkdownCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};
use syn::{self, Data, DataEnum, DeriveInput, Fields, Ident, PathSegment, Type, parse_macro_input};

use common::{
    inflector::Inflector,
    proc_macro2::{Span, TokenStream},
    quote::quote,
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
}

#[derive(FromField)]
#[darling(attributes(markdown))]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,
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

    if let Some(template) = type_attr.template {
        // If a Markdown template is provided, render each of the interpolated strings
        // and push the other content between
        let mut fields = TokenStream::new();
        let mut included = vec![];
        for segment in template
            .split_inclusive("}}")
            .flat_map(|segment| segment.split("{{"))
        {
            if let Some(field_name) = segment.strip_suffix("}}") {
                let property =
                    Ident::new(&field_name.to_string().to_pascal_case(), Span::call_site());
                let field_ident = Ident::new(field_name, Span::call_site());
                let tokens = if let Some(escape) = &type_attr.escape {
                    quote! {
                        context
                            .set_escape(#escape)
                            .push_prop_fn(NodeProperty::#property, |context| self.#field_ident.to_markdown(context))
                            .clear_escape()
                        ;
                    }
                } else {
                    quote! {
                        context.push_prop_fn(NodeProperty::#property, |context| self.#field_ident.to_markdown(context));
                    }
                };
                fields.extend(tokens);
                included.push(field_name);
            } else {
                fields.extend(quote! {
                    context.push_str(#segment);
                });
            }
        }

        // Create tokens for losses
        let mut losses = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return;
            };
            if field_name == "r#type"
                || field_name == "uid"
                || field_name == "options" // Ignore options!
                || included.contains(&field_name.to_string().as_str())
            {
                return;
            }
            let Type::Path(type_path) = field_attr.ty else {
                return;
            };
            let Some(PathSegment {
                ident: field_type, ..
            }) = type_path.path.segments.last()
            else {
                return;
            };

            let record_loss = quote! {
                context.add_loss(concat!(stringify!(#struct_name), ".", stringify!(#field_name)));
            };

            let loss = if field_type == "Option" {
                quote! { if self.#field_name.is_some() { #record_loss }}
            } else if field_type == "Vec" {
                quote! { if !self.#field_name.is_empty() { #record_loss }}
            } else {
                record_loss
            };

            losses.extend(loss);
        });

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
                    context.enter_node(self.node_type(), self.node_id());
                    #fields
                    #losses
                    context.exit_node();
                }
            }
        }
    } else {
        // If no Markdown template provided, encode all fields but record loss of
        // structure of this type  (but not for XxxxOptions)

        let (enter, exit) = if struct_name.to_string().ends_with("Options") {
            (TokenStream::new(), TokenStream::new())
        } else {
            (
                quote!(
                    context.enter_node(self.node_type(), self.node_id()).add_loss(concat!(stringify!(#struct_name), "#"));
                ),
                quote!(
                    context.exit_node();
                ),
            )
        };

        let mut fields = TokenStream::new();
        type_attr.data.map_struct_fields(|field_attr| {
            let Some(field_name) = field_attr.ident else {
                return
            };

            if field_name == "r#type" || field_name == "uid" || field_name == "options" {
                return;
            }

            let property = if field_name == "r#abstract" {
                Ident::new("Abstract", Span::call_site())
            } else {
                Ident::new(&field_name.to_string().to_pascal_case(), Span::call_site())
            };
            let field_tokens = quote! {
                context.push_prop_fn(NodeProperty::#property, |context| self.#field_name.to_markdown(context));
            };
            fields.extend(field_tokens)
        });

        quote! {
            impl MarkdownCodec for #struct_name {
                fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
                    #enter
                    #fields
                    #exit
                }
            }
        }
    }
}

/// Derive the `MarkdownCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let content_type = if enum_name == "Block" {
        quote! {Some(ContentType::Block)}
    } else if enum_name == "Inline" {
        quote! {Some(ContentType::Inline)}
    } else {
        quote! {None}
    };

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_tokens = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.to_markdown(context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.push_str(stringify!(#variant_name)); },
            },
        };
        variants.extend(variant_tokens)
    }

    quote! {
        impl MarkdownCodec for #enum_name {
            fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
                context.content_type = #content_type;
                match self {
                    #variants
                }
                context.content_type = None;
            }
        }
    }
}
