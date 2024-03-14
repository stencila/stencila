//! Provides the `DomCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::TokenStream,
    quote::quote,
    syn::{
        parse_macro_input, parse_str, Data, DataEnum, DeriveInput, Fields, Ident, Path,
        PathSegment, Type,
    },
};

#[derive(FromDeriveInput)]
#[darling(attributes(dom))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,
}

#[derive(FromField)]
#[darling(attributes(dom))]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    elem: Option<String>,

    #[darling(default)]
    attr: Option<String>,

    #[darling(default)]
    with: Option<String>,
}

/// Derive the `DomCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(DomCodec, attributes(dom))]
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

/// Derive the `DomCodec` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    let (enter, exit) = if struct_name.to_string().ends_with("Options") {
        (TokenStream::new(), TokenStream::new())
    } else {
        (
            quote!(
                context.enter_node(self.node_type(), self.node_id());
            ),
            quote!(
                context.exit_node();
            ),
        )
    };

    let mut attrs = TokenStream::new();
    // Options may include attrs so these need to go directly after the "main" attrs
    let mut options = TokenStream::new();
    let mut children = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return
        };

        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        if field_name == "id" {
            attrs.extend(quote! {
                context.push_id(&self.id);
            });
        } else if field_name == "options" {
            options.extend(quote! {
                self.#field_name.to_dom(context);
            });
        } else if field_attr.skip {
            return;
        } else if let Some(with) = field_attr.with.as_deref() {
            let Type::Path(type_path) = field_attr.ty else {
                return
            };
            let Some(PathSegment{ident: field_type,..}) = type_path.path.segments.last() else {
                return
            };

            let func = parse_str::<Path>(with).expect("invalid DOM `with` option");

            let tokens = if field_type == "Option" {
                quote! {
                    if let Some(value) = self.#field_name.as_ref() {
                        #func(stringify!(#field_name), value, context);
                    }
                }
            } else {
                quote! {
                    #func(stringify!(#field_name), &self.#field_name, context);
                }
            };

            attrs.extend(tokens);
        } else if matches!(field_attr.elem.as_deref(), Some("none")) {
            children.extend(quote! {
                self.#field_name.to_dom(context);
            });
        } else if let Some(elem) = field_attr.elem {
            let Type::Path(type_path) = field_attr.ty else {
                return
            };
            let Some(PathSegment{ident: field_type,..}) = type_path.path.segments.last() else {
                return
            };


            let tokens = if  field_type == "Cord" || field_type == "String" 
            {
                // Avoid having <stencila-string> wrapping text for the few `Cord` and `String` properties represented as slots
                quote! {
                    context.enter_slot(#elem, stringify!(#field_name)).push_text(&self.#field_name).exit_slot();
                }
            } else if struct_name == "ExecutionMessage" && field_name == "stack_trace" {
                // Avoid having <stencila-string> wrapping text for the few `Option<String>` properties represented as slots
                quote! {
                    if let Some(text) = &self.#field_name {
                        context.enter_slot(#elem, stringify!(#field_name)).push_text(text).exit_slot();
                    }
                }
            } else {
                quote! {
                    context.push_slot_fn(#elem, stringify!(#field_name), |context| self.#field_name.to_dom(context));
                }
            };

            let tokens = if field_type == "Option" {
                quote! { if self.#field_name.is_some() { #tokens }}
            } else if field_type == "Vec" {
                quote! { if !self.#field_name.is_empty() { #tokens }}
            } else {
                tokens
            };

            children.extend(tokens);
        } else {
            let attr_name = field_attr
                .attr
                .unwrap_or_else(|| field_name.to_string().to_kebab_case());

            attrs.extend(quote! {
                self.#field_name.to_dom_attr(#attr_name, context);
            });
        }
    });

    quote! {
        impl DomCodec for #struct_name {
            fn to_dom(&self, context: &mut DomEncodeContext) {
                #enter
                #attrs
                #options
                #children
                #exit
            }
        }
    }
}

/// Derive the `DomCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut to_dom = TokenStream::new();
    let mut to_dom_attr = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        to_dom.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.to_dom(context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.push_text(stringify!(#variant_name)); },
            },
        });

        to_dom_attr.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.to_dom_attr(name, context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.push_attr(name, stringify!(#variant_name)); },
            },
        });
    }

    quote! {
        impl DomCodec for #enum_name {
            fn to_dom(&self, context: &mut DomEncodeContext) {
                match self {
                    #to_dom
                }
            }

            fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
                match self {
                    #to_dom_attr
                }
            }
        }
    }
}
