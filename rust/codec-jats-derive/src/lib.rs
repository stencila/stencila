//! Provides a `JatsCodec` derive macro for structs and enums in Stencila Schema

use std::collections::HashMap;

use darling::{FromDeriveInput, FromField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, Ident, PathSegment, Type, parse_macro_input};

#[derive(FromDeriveInput)]
#[darling(attributes(jats))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    elem: Option<String>,

    #[darling(default)]
    attribs: HashMap<String, String>,
}

#[derive(FromField)]
#[darling(attributes(jats))]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    elem: Option<String>,

    #[darling(default)]
    attr: Option<String>,

    #[darling(default)]
    content: bool,

    #[darling(default)]
    flatten: bool,
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

    let (enter_elem, exit_elem) = if let Some(elem) = type_attr.elem {
        (
            quote! { context.enter_elem(#elem); },
            quote! { context.exit_elem(); },
        )
    } else if struct_name.to_string().ends_with("Options") {
        (TokenStream::new(), TokenStream::new())
    } else {
        return quote! {
            impl JatsCodec for #struct_name {
                fn to_jats(&self, context: &mut JatsEncodeContext) {
                    context.add_loss(stringify!(#struct_name));
                }
            }
        };
    };

    let mut attrs = TokenStream::new();
    for (name, value) in type_attr.attribs.iter().sorted() {
        let name = name.replace("__", "-").replace('_', ":");
        attrs.extend(quote! {
            context.push_attr(#name, #value);
        })
    }

    let mut field_attrs = TokenStream::new();
    let mut options = TokenStream::new();
    let mut children = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return;
        };
        let is_flatten = field_attr.flatten;
        let is_attr = field_attr.attr.is_some();

        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        let field_tokens = if field_attr.flatten {
            quote! {
                self.#field_name.to_jats(context);
            }
        } else if let Some(attr) = field_attr.attr {
            quote! {
                let field_text = self.#field_name.to_text();
                if !field_text.is_empty() {
                    context.push_attr(#attr, field_text);
                }
            }
        } else if let Some(elem) = field_attr.elem {
            quote! {
                context.enter_elem(#elem);
                self.#field_name.to_jats(context);
                context.exit_elem_omit_empty();
            }
        } else if field_name == "content" || field_attr.content {
            quote! {
                self.#field_name.to_jats(context);
            }
        } else {
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

            if field_name == "label_automatically" {
                // Do not record loss for derived properties
                quote!()
            } else if field_type == "Option" {
                quote! { if self.#field_name.is_some() { #record_loss }}
            } else if field_type == "Vec" {
                quote! { if !self.#field_name.is_empty() { #record_loss }}
            } else {
                record_loss
            }
        };
        if is_flatten {
            options.extend(field_tokens)
        } else if is_attr {
            field_attrs.extend(field_tokens)
        } else {
            children.extend(field_tokens)
        }
    });

    quote! {
        impl JatsCodec for #struct_name {
            fn to_jats(&self, context: &mut JatsEncodeContext) {
                #enter_elem
                #attrs
                #field_attrs
                #options
                #children
                #exit_elem
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
                    Self::#variant_name(v) => v.to_jats(context),
                });
            }
            Fields::Unit => {
                variants_to_jats.extend(quote! {
                    Self::#variant_name => { context.push_text(stringify!(#variant_name)); },
                });
            }
        };
    }

    quote! {
        impl JatsCodec for #enum_name {
            fn to_jats(&self, context: &mut JatsEncodeContext) {
                match self {
                    #variants_to_jats
                }
            }
        }
    }
}
