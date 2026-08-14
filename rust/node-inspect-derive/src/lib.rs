//! Provides an `InspectNode` derive macro for structs and enums in Stencila Schema
//!
//! The macro derives both the `InspectType` trait (the static, type-level view of
//! what the schema declares) and the `InspectNode` trait (the borrowed, value-level
//! view). Both traits are defined in the `stencila-schema` crate.
//!
//! The macro is deliberately free of any comparison policy: it reports only what the
//! schema declares.

use darling::{FromDeriveInput, FromField};
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Data, DataEnum, DeriveInput, Fields, GenericArgument, Ident, PathArguments, Type,
    parse_macro_input,
};

#[derive(FromDeriveInput)]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,
}

#[derive(FromField)]
struct FieldAttr {
    ident: Option<Ident>,
    ty: Type,
}

/// Derive the `InspectType` and `InspectNode` traits for a `struct` or `enum`
#[proc_macro_derive(InspectNode)]
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

/// The name of the `NodeProperty` variant for a field
fn node_property(field_name: &Ident) -> Ident {
    let name = field_name.to_string();
    // Field names are snake case, so pascal casing them yields the same variant name
    // that the generator derives from the schema's camel case property name
    let property = if name == "r#abstract" {
        "Abstract".to_string()
    } else {
        name.to_pascal_case()
    };
    Ident::new(&property, Span::call_site())
}

/// If `ty` is `Wrapper<Inner>` for the given wrapper name, return `Inner`
fn unwrap_type<'ty>(ty: &'ty Type, wrapper: &str) -> Option<&'ty Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != wrapper {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    })
}

/// The schema-level shape of a field: optionality, cardinality, and the type
/// that carries the value kind
struct FieldShape<'ty> {
    required: bool,
    repeated: bool,
    /// The type used to determine the value kind, with `Option` and `Vec` peeled off
    /// (`Box` is left in place because its `InspectType` impl delegates)
    inner: &'ty Type,
}

fn field_shape(ty: &Type) -> FieldShape<'_> {
    let (required, ty) = match unwrap_type(ty, "Option") {
        Some(inner) => (false, inner),
        None => (true, ty),
    };
    let (repeated, ty) = match unwrap_type(ty, "Vec") {
        Some(inner) => (true, inner),
        None => (false, ty),
    };
    FieldShape {
        required,
        repeated,
        inner: ty,
    }
}

/// Derive for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;
    let name = struct_name.to_string();

    // `*Options` structs are flattened property containers: they are not occurrences
    // of their own and their properties are reported by their owning type
    let is_options = name.ends_with("Options");

    let mut decls = TokenStream::new();
    let mut props = TokenStream::new();
    let mut has_options_field = false;

    type_attr.data.map_struct_fields(|field| {
        let Some(field_name) = field.ident else {
            return;
        };

        // Intrinsic implementation machinery, not schema properties. The type
        // discriminator is reported structurally as the `NodeType` instead.
        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        // The `options` field is flattened into the owning type
        if field_name == "options" && !is_options {
            has_options_field = true;
            return;
        }

        let property = node_property(&field_name);
        let FieldShape {
            required,
            repeated,
            inner,
        } = field_shape(&field.ty);

        let decl = quote! {
            PropertyDecl {
                property: NodeProperty::#property,
                required: #required,
                repeated: #repeated,
                kind: <#inner as InspectType>::VALUE_KIND,
            }
        };

        let value = match (required, repeated) {
            (true, false) => quote! {
                InspectValue::One(&self.#field_name)
            },
            (true, true) => quote! {
                InspectValue::Many(
                    self.#field_name.iter().map(|item| item as &dyn InspectNode).collect()
                )
            },
            (false, false) => quote! {
                match &self.#field_name {
                    Some(value) => InspectValue::One(value),
                    None => InspectValue::Absent,
                }
            },
            (false, true) => quote! {
                match &self.#field_name {
                    Some(values) => InspectValue::Many(
                        values.iter().map(|item| item as &dyn InspectNode).collect()
                    ),
                    None => InspectValue::Absent,
                }
            },
        };

        decls.extend(quote! {
            decls.push(#decl);
        });
        props.extend(quote! {
            props.push(InspectProperty { decl: #decl, value: #value });
        });
    });

    // The type of the `options` field, needed to flatten its declarations
    let (options_decls, options_props) = if has_options_field {
        let options_type = Ident::new(&format!("{name}Options"), Span::call_site());
        (
            quote! {
                decls.extend(<#options_type as InspectType>::declared_properties());
            },
            quote! {
                props.extend(InspectNode::properties(&self.options));
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    let (value_kind, node_type) = if is_options {
        (
            quote!(ValueKind::Flattened),
            quote! {
                fn node_type(&self) -> Option<NodeType> {
                    None
                }
            },
        )
    } else {
        (
            quote!(ValueKind::Structured),
            quote! {
                fn node_type(&self) -> Option<NodeType> {
                    Some(NodeType::#struct_name)
                }
            },
        )
    };

    quote! {
        impl InspectType for #struct_name {
            const VALUE_KIND: ValueKind = #value_kind;

            fn declared_properties() -> Vec<PropertyDecl> {
                let mut decls = Vec::new();
                #decls
                #options_decls
                decls
            }
        }

        impl InspectNode for #struct_name {
            #node_type

            fn properties(&self) -> Vec<InspectProperty<'_>> {
                let mut props = Vec::new();
                #props
                #options_props
                props
            }
        }
    }
}

/// Derive for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;
    let enum_string = enum_name.to_string();

    let mut has_typed_variant = false;
    let mut node_type_arms = TokenStream::new();
    let mut properties_arms = TokenStream::new();
    let mut scalar_arms = TokenStream::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => {
                has_typed_variant = true;
                node_type_arms.extend(quote! {
                    Self::#variant_name(variant) => InspectNode::node_type(variant),
                });
                properties_arms.extend(quote! {
                    Self::#variant_name(variant) => InspectNode::properties(variant),
                });
                scalar_arms.extend(quote! {
                    Self::#variant_name(variant) => InspectNode::scalar(variant),
                });
            }
            Fields::Unit => {
                let variant_string = variant_name.to_string();
                node_type_arms.extend(quote! {
                    Self::#variant_name => None,
                });
                properties_arms.extend(quote! {
                    Self::#variant_name => Vec::new(),
                });
                scalar_arms.extend(quote! {
                    Self::#variant_name => Some(ScalarRef::Enum {
                        schema_type: #enum_string,
                        variant: #variant_string,
                    }),
                });
            }
        }
    }

    // A union enum is a transparent wrapper around its selected branch; an enum with
    // only unit variants is a schema enum, which is a scalar value
    let value_kind = if has_typed_variant {
        quote!(ValueKind::Union)
    } else {
        quote!(ValueKind::Scalar)
    };

    quote! {
        impl InspectType for #enum_name {
            const VALUE_KIND: ValueKind = #value_kind;
        }

        impl InspectNode for #enum_name {
            fn node_type(&self) -> Option<NodeType> {
                match self {
                    #node_type_arms
                }
            }

            fn properties(&self) -> Vec<InspectProperty<'_>> {
                match self {
                    #properties_arms
                }
            }

            fn scalar(&self) -> Option<ScalarRef<'_>> {
                match self {
                    #scalar_arms
                }
            }
        }
    }
}
