//! Provides the `PatchNode` derive macro for structs and enums in Stencila Schema

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

/// Derive the `PatchNode` trait for a `struct` or an `enum`
#[proc_macro_derive(PatchNode, attributes(merge))]
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

/// Derive the `PatchNode` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    let (condense_enter, condense_exit) = if struct_name.to_string().ends_with("Options") {
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

    let mut condense_fields = TokenStream::new();
    let mut get_fields = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return;
        };

        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        if field_name == "options" {
            condense_fields.extend(quote! {
                self.#field_name.condense(context);
            });
        } else if !field_attr.formats.is_empty() {
            // TODO: This currently does not consider different formats. Merging
            // is turned on for the property if there is any format in the list.
            let property = field_name.to_string().to_pascal_case();
            let property = Ident::new(&property, Span::call_site());
            condense_fields.extend(quote! {
                context.enter_property(NodeProperty::#property);
                self.#field_name.condense(context);
                context.exit_property();
            });
        }
    });

    quote! {
        impl PatchNode for #struct_name {
            fn condense(&self, context: &mut CondenseContext) {
                #condense_enter
                #condense_fields
                #condense_exit
            }

            fn get_path(&self, path: &mut NodePath) -> Result<Value> {
                if path.is_empty() {
                    Ok(serde_json::to_value(self)?)
                } else {
                    todo!()
                }
            }
        }
    }
}

/// Derive the `PatchNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut condense_variants = TokenStream::new();
    let mut get_variants = TokenStream::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;

        condense_variants.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.condense(context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.collect_value(stringify!(#variant_name)); },
            },
        });

        get_variants.extend(match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => variant.get_path(path),
            },
            Fields::Unit => quote! {
                Self::#variant_name => Ok(serde_json::to_value(stringify!(#variant_name))?),
            },
        });
    }

    quote! {
        impl PatchNode for #enum_name {
            fn condense(&self, context: &mut CondenseContext) {
                match self {
                    #condense_variants
                }
            }

            fn get_path(&self, path: &mut NodePath) -> Result<Value> {
                match self {
                    #get_variants
                }
            }
        }
    }
}
