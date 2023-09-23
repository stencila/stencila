//! Provides a `StripNode` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident, PathSegment, Type},
};

#[derive(FromDeriveInput)]
#[darling(attributes(strip))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,
}

#[derive(FromField)]
#[darling(attributes(strip))]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    id: bool,
    #[darling(default)]
    code: bool,
    #[darling(default)]
    execution: bool,
    #[darling(default)]
    output: bool,
    #[darling(default)]
    types: bool,
}

/// Derive the `StripNode` trait for a `struct` or `enum`
#[proc_macro_derive(StripNode, attributes(strip))]
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

/// Derive the `StripNode` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    let mut fields = TokenStream::new();
    type_attr.data.map_struct_fields(|field| {
        let field_name = field.ident;

        let Type::Path(type_path) = field.ty else {
            return
        };
        let Some(PathSegment{ident: field_type,..}) = type_path.path.segments.last() else {
           return
        };

        // The tokens needed to strip the field
        let strip = if field_type == "Option" {
            quote! { = None; }
        } else {
            quote! { .clear() }
        };

        // Strip the field if it is targeted

        if field.id {
            fields.extend(quote! {
                if targets.id {
                    self.#field_name #strip;
                }
            })
        }

        if field.code {
            fields.extend(quote! {
                if targets.code {
                    self.#field_name #strip;
                }
            })
        }

        if field.execution {
            fields.extend(quote! {
                if targets.execution {
                    self.#field_name #strip;
                }
            })
        }

        if field.output {
            fields.extend(quote! {
                if targets.output {
                    self.#field_name #strip;
                }
            })
        }

        if field.types {
            let tokens = if field_type == "Option" {
                quote! {
                    if let Some(children) = self.#field_name.as_mut() {
                        children.retain(|child| !targets.types.contains(&child.to_string()));
                    }
                }
            } else {
                quote! {
                    self.#field_name.retain(|child| !targets.types.contains(&child.to_string()));
                }
            };
            fields.extend(tokens)
        }

        // For all fields, recursively call strip
        fields.extend(quote! {
            self.#field_name.strip(targets);
        })
    });

    quote! {
        impl node_strip::StripNode for #struct_name {
            fn strip(&mut self, targets: &node_strip::Targets) -> &mut Self {
                #fields
                self
            }
        }
    }
}

/// Derive the `StripNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => { v.strip(targets); },
            },
            Fields::Unit => continue,
        };
        variants.extend(case)
    }

    if variants.is_empty() {
        quote! {
            impl node_strip::StripNode for #enum_name {}
        }
    } else {
        quote! {
            impl node_strip::StripNode for #enum_name {
                fn strip(&mut self, targets: &node_strip::Targets) -> &mut Self {
                    match self {
                        #variants
                    }
                    self
                }
            }
        }
    }
}
