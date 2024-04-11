//! Provides the `CondenseNode` derive macro for structs and enums in Stencila Schema

use darling::{self, ast::Data as AstData, util::Ignored, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
#[darling(attributes(merge))]
struct TypeAttr {
    ident: Ident,
    data: AstData<Ignored, FieldAttr>,
}

#[derive(FromField)]
#[darling(attributes(merge))]
struct FieldAttr {
    ident: Option<Ident>,

    #[darling(multiple, rename = "format")]
    formats: Vec<String>,
}

/// Derive the `CondenseNode` trait for a `struct` or an `enum`
#[proc_macro_derive(CondenseNode, attributes(merge))]
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

/// Derive the `CondenseNode` trait for a `struct`
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

    let mut properties = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return;
        };

        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        if field_name == "options" {
            properties.extend(quote! {
                self.#field_name.condense(context);
            });
        } else if !field_attr.formats.is_empty() {
            // TODO: This currently does not consider different formats. Merging
            // is turned on for the property if there is any format in the list.
            let property = field_name.to_string().to_pascal_case();
            let property = Ident::new(&property, Span::call_site());
            properties.extend(quote! {
                context.enter_property(NodeProperty::#property);
                self.#field_name.condense(context);
                context.exit_property();
            });
        }
    });

    quote! {
        impl CondenseNode for #struct_name {
            fn condense(&self, context: &mut CondenseContext) {
                #enter
                #properties
                #exit
            }
        }
    }
}

/// Derive the `CondenseNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut condense = TokenStream::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;

        condense.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.condense(context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.collect_value(stringify!(#variant_name)); },
            },
        });
    }

    quote! {
        impl CondenseNode for #enum_name {
            fn condense(&self, context: &mut CondenseContext) {
                match self {
                    #condense
                }
            }
        }
    }
}
