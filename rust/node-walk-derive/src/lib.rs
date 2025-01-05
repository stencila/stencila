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

    let (enter, enter_async, exit) = if !struct_name.to_string().ends_with("Options") {
        (
            quote! {
                if visitor.enter_struct(self.node_type(), self.node_id()).is_break() {
                    return
                }
            },
            quote! {
                if visitor.enter_struct(self.node_type(), self.node_id()).is_break() {
                    return Ok(())
                }
            },
            quote! {
                visitor.exit_struct();
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new(), TokenStream::new())
    };

    let (visit, visit_mut, visit_async) = {
        let method = match struct_name.to_string().as_str() {
            "IfBlockClause" => Some(quote!(visit_if_block_clause)),
            "ListItem" => Some(quote!(visit_list_item)),
            "SuggestionBlock" => Some(quote!(visit_suggestion_block)),
            "SuggestionInline" => Some(quote!(visit_suggestion_inline)),
            "TableRow" => Some(quote!(visit_table_row)),
            "TableCell" => Some(quote!(visit_table_cell)),
            "WalkthroughStep" => Some(quote!(visit_walkthrough_step)),
            _ => None,
        };

        if let Some(method) = method {
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
        } else {
            (TokenStream::new(), TokenStream::new(), TokenStream::new())
        }
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
                if visitor.enter_property(NodeProperty::#property).is_break() {
                    return
                }

                self.#field_name.walk(visitor);

                visitor.exit_property();
            });
            fields_mut.extend(quote! {
                if visitor.enter_property(NodeProperty::#property).is_break() {
                    return
                }

                self.#field_name.walk_mut(visitor);

                visitor.exit_property();
            });
            fields_async.extend(quote! {
                if visitor.enter_property(NodeProperty::#property).is_break() {
                    return Ok(())
                }

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
                    #enter_async
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

    let (visit, visit_mut, visit_async) = {
        let method = match enum_name.to_string().as_str() {
            "Node" => Some(quote!(visit_node)),
            "CreativeWorkType" => Some(quote!(visit_work)),
            "Block" => Some(quote!(visit_block)),
            "Inline" => Some(quote!(visit_inline)),
            _ => None,
        };

        if let Some(method) = method {
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
        } else {
            (TokenStream::new(), TokenStream::new(), TokenStream::new())
        }
    };

    let mut variants = TokenStream::new();
    let mut variants_mut = TokenStream::new();
    let mut variants_async = TokenStream::new();
    for variant in &data.variants {
        if matches!(variant.fields, Fields::Named(..) | Fields::Unnamed(..)) {
            let variant_name = &variant.ident;
            variants.extend(quote! {
                Self::#variant_name(variant) => variant.walk(visitor),
            });
            variants_mut.extend(quote! {
                Self::#variant_name(variant) => variant.walk_mut(visitor),
            });
            variants_async.extend(quote! {
                Self::#variant_name(variant) => variant.walk_async(visitor).await?,
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
