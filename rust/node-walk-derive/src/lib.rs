//! Provides a `WalkNode` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,
}

#[derive(FromField)]
#[darling(forward_attrs(walk))]
struct FieldAttr {
    ident: Option<Ident>,
    attrs: Vec<Attribute>,
}

/// Derive the `WalkNode` trait for a `struct` or `enum`
#[proc_macro_derive(WalkNode, attributes(walk))]
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

/// Derive the `WalkNode` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    let (enter, exit) = if !struct_name.to_string().ends_with("Options") {
        (
            quote! {
                visitor.enter_struct(self.node_type(), self.node_id());
            },
            quote! {
                visitor.exit_struct();
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    let (visit, visit_mut, visit_async) = match struct_name.to_string().as_str() {
        name @ ("IfBlockClause" | "ListItem" | "TableRow" | "TableCell") => {
            let method = match name {
                "IfBlockClause" => quote!(visit_if_block_clause),
                "ListItem" => quote!(visit_list_item),
                "TableRow" => quote!(visit_table_row),
                "TableCell" => quote!(visit_table_cell),
                _ => unreachable!(),
            };

            (
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method(self).await?.is_break() {
                        return Ok(())
                    }
                },
            )
        }
        _ => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
    };

    let mut fields = TokenStream::new();
    let mut fields_mut = TokenStream::new();
    let mut fields_async = TokenStream::new();
    type_attr.data.map_struct_fields(|field| {
        if !field.attrs.is_empty() {
            let Some(field_name) = field.ident else {
                return;
            };
            let property = Ident::new(&field_name.to_string().to_pascal_case(), Span::call_site());

            fields.extend(quote! {
                visitor.enter_property(NodeProperty::#property);
                self.#field_name.walk(visitor);
                visitor.exit_property();
            });
            fields_mut.extend(quote! {
                visitor.enter_property(NodeProperty::#property);
                self.#field_name.walk_mut(visitor);
                visitor.exit_property();
            });
            fields_async.extend(quote! {
                visitor.enter_property(NodeProperty::#property);
                self.#field_name.walk_async(visitor).await?;
                visitor.exit_property();
            })
        }
    });

    if fields.is_empty() {
        quote! {
            impl WalkNode for #struct_name {}
        }
    } else {
        quote! {
            impl WalkNode for #struct_name {
                fn walk<V: Visitor>(&self, visitor: &mut V) {
                    #enter
                    #visit
                    #fields
                    #exit
                }

                fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
                    #enter
                    #visit_mut
                    #fields_mut
                    #exit
                }

                #[async_recursion]
                async fn walk_async<V: VisitorAsync>(&mut self, visitor: &mut V) -> Result<()> {
                    #enter
                    #visit_async
                    #fields_async
                    #exit
                    Ok(())
                }
            }
        }
    }
}

/// Derive the `WalkNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let (visit, visit_mut, visit_async) = match enum_name.to_string().as_str() {
        name @ ("Node" | "CreativeWorkType" | "Block" | "Inline") => {
            let method = match name {
                "Node" => quote!(visit_node),
                "CreativeWorkType" => quote!(visit_work),
                "Block" => quote!(visit_block),
                "Inline" => quote!(visit_inline),
                _ => unreachable!(),
            };

            (
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method(self).await?.is_break() {
                        return Ok(())
                    }
                },
            )
        }
        _ => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
    };

    let mut variants = TokenStream::new();
    let mut variants_mut = TokenStream::new();
    let mut variants_async = TokenStream::new();
    for variant in &data.variants {
        if matches!(variant.fields, Fields::Named(..) | Fields::Unnamed(..)) {
            let variant_name = &variant.ident;
            variants.extend(quote! {
                Self::#variant_name(variant) => { variant.walk(visitor); },
            });
            variants_mut.extend(quote! {
                Self::#variant_name(variant) => { variant.walk_mut(visitor); },
            });
            variants_async.extend(quote! {
                Self::#variant_name(variant) => { variant.walk_async(visitor).await?; },
            })
        };
    }

    if variants.is_empty() {
        quote! {
            impl WalkNode for #enum_name {}
        }
    } else {
        quote! {
            impl WalkNode for #enum_name {
                fn walk<V: Visitor>(&self, visitor: &mut V) {
                    #visit;
                    match self {
                        #variants
                    }
                }

                fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
                    #visit_mut;
                    match self {
                        #variants_mut
                    }
                }

                #[async_recursion]
                async fn walk_async<V: VisitorAsync>(&mut self, visitor: &mut V) -> Result<()> {
                    #visit_async;
                    match self {
                        #variants_async
                    }
                    Ok(())
                }
            }
        }
    }
}
