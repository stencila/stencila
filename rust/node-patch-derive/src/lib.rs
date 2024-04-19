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

    let mut similarity = TokenStream::new();
    let mut diff = TokenStream::new();
    let mut patch = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return;
        };
        if field_name == "r#type" || field_name == "uid" || field_name == "options" {
            return;
        }

        if !field_attr.formats.is_empty() {
            // TODO: This currently does not consider different formats. Merging
            // is turned on for the property if there is any format in the list.

            let property = Ident::new(&field_name.to_string().to_pascal_case(), Span::call_site());

            similarity.extend(quote! {
                self.#field_name.similarity(&other.#field_name, context)?,
            });
            diff.extend(quote! {
                context.enter_property(NodeProperty::#property);
                self.#field_name.diff(&other.#field_name, context)?;
                context.exit_property();
            });
            patch.extend(quote! {
                NodeProperty::#property => self.#field_name.patch(path, op, context),
            });
        }
    });

    quote! {
        impl PatchNode for #struct_name {
            fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
                PatchContext::mean_similarity(vec![
                    #similarity
                ])
            }

            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                #diff

                Ok(())
            }

            fn patch(&mut self, path: &mut PatchPath, op: PatchOp, context: &mut PatchContext) -> Result<()> {
                let Some(PatchSlot::Property(property)) = path.pop_front() else {
                    bail!("Invalid patch path for `{}`", stringify!(#struct_name));
                };

                match (property) {
                    #patch
                    _ => bail!("Invalid property for `{}`", stringify!(#struct_name))
                }
            }
        }
    }
}

/// Derive the `PatchNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let (to_value, from_value) = match enum_name.to_string().as_str() {
        "Inline" | "Block" | "Node" => (
            quote! {
                Ok(PatchValue::#enum_name(self.clone()))
            },
            quote! {
                match value {
                    PatchValue::#enum_name(value) => Ok(value),
                    _ => bail!("Invalid value for `{}`", stringify!(#enum_name))
                }
            },
        ),
        _ => (
            quote! {
                Ok(PatchValue::Json(serde_json::to_value(self)?))
            },
            quote! {
                match value {
                    PatchValue::Json(json) => Ok(serde_json::from_value(json)?),
                    _ => bail!("Invalid patch value for `{}`", stringify!(#enum_name))
                }
            },
        ),
    };

    let mut similarity = TokenStream::new();
    let mut diff = TokenStream::new();
    let mut patch = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => {
                similarity.extend(quote! {
                    (Self::#variant_name(me), Self::#variant_name(other)) => me.similarity(other, context),
                });
                diff.extend(quote! {
                    (Self::#variant_name(me), Self::#variant_name(other)) => me.diff(other, context),
                });
                patch.extend(quote! {
                    Self::#variant_name(me) => me.patch(path, op, context),
                });
            }
            Fields::Unit => {
                similarity.extend(quote! {
                    (Self::#variant_name, Self::#variant_name) => Ok(1.0),
                });
                diff.extend(quote! {
                    (Self::#variant_name, Self::#variant_name) => Ok(()),
                });
                patch.extend(quote! {
                    Self::#variant_name => Ok(()),
                });
            }
        };
    }

    quote! {
        impl PatchNode for #enum_name {
            fn to_value(&self) -> Result<PatchValue> {
                #to_value
            }

            fn from_value(value: PatchValue) -> Result<Self> {
                #from_value
            }

            fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
                match (self, other) {
                    // Same variants
                    #similarity
                    // Different variants: zero similarity
                    _ => Ok(0.0)
                }
            }

            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                match (self, other) {
                    // Same variants
                    #diff
                    // Different variants: set with other
                    _ => {
                        context.op_set(other.to_value()?);
                        Ok(())
                    }
                }
            }

            fn patch(&mut self, path: &mut PatchPath, op: PatchOp, context: &mut PatchContext) -> Result<()> {
                match self {
                    #patch
                }
            }
        }
    }
}
