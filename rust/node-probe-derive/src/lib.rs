//! Provides a `ProbeNode` derive macro for structs and enums in Stencila Schema

use darling::{FromDeriveInput, FromField};
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, Ident, parse_macro_input};

#[derive(FromDeriveInput)]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,
}

#[derive(FromField)]
struct FieldAttr {
    ident: Option<Ident>,
}

/// Derive the `ProbeNode` trait for a `struct` or `enum`
#[proc_macro_derive(ProbeNode)]
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

/// Derive the `ProbeNode` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    let mut fields = TokenStream::new();
    let mut has_options = false;
    type_attr.data.map_struct_fields(|field| {
        let Some(field_name) = field.ident else {
            return;
        };
        if field_name == "r#type" || field_name == "uid" {
            return;
        }
        if field_name == "options" {
            has_options = true;
            return;
        }

        let property = if field_name == "r#abstract" {
            "Abstract".to_string()
        } else {
            field_name.to_string().to_pascal_case()
        };
        let property = Ident::new(&property, Span::call_site());

        fields.extend(quote! {
            NodeProperty::#property => self.#field_name.duplicate(path),
        });
    });

    let empty_path = if !struct_name.to_string().ends_with("Options")
        && !struct_name.to_string().starts_with("Config")
    {
        quote! {
            if path.is_empty() {
                return Ok(NodeSet::One(Node::#struct_name(self.clone())));
            }
        }
    } else {
        quote!()
    };

    let no_match = if has_options {
        quote! {
            self.options.duplicate(path)
        }
    } else {
        quote! {
            bail!("Invalid property {property}")
        }
    };

    quote! {
        impl ProbeNode for #struct_name {
            fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
                #empty_path

                let Some(NodeSlot::Property(property)) = path.pop_front() else {
                    bail!("Node path should have property at front for struct")
                };

                match property {
                    #fields
                    _ => #no_match
                }
            }
        }
    }
}

/// Derive the `ProbeNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        variants.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => variant.duplicate(path),
            },
            Fields::Unit => continue,
        })
    }

    if variants.is_empty() {
        quote! {
            impl ProbeNode for #enum_name {
                fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
                    Ok(NodeSet::One(Node::String(self.to_string())))
                }
            }
        }
    } else {
        quote! {
            impl ProbeNode for #enum_name {
                fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
                    match self {
                        #variants
                    }
                }
            }
        }
    }
}
