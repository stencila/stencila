//! Provides a `WalkNode` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    proc_macro2::TokenStream,
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

    let (visit, visit_mut) = match struct_name.to_string().as_str() {
        name @ ("ListItem" | "TableRow" | "TableCell") => {
            let (method, method_mut) = match name {
                "ListItem" => (quote!(visit_list_item), quote!(visit_list_item_mut)),
                "TableRow" => (quote!(visit_table_row), quote!(visit_table_row_mut)),
                "TableCell" => (quote!(visit_table_cell), quote!(visit_table_cell_mut)),
                _ => unreachable!(),
            };

            (
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method_mut(self).is_break() {
                        return
                    }
                },
            )
        }
        _ => (TokenStream::new(), TokenStream::new()),
    };

    let mut fields = TokenStream::new();
    let mut fields_mut = TokenStream::new();
    type_attr.data.map_struct_fields(|field| {
        if !field.attrs.is_empty() {
            let field_name = field.ident;
            fields.extend(quote! {
                visitor.enter_property(stringify!(#field_name));
                self.#field_name.walk(visitor);
                visitor.exit_property();
            });
            fields_mut.extend(quote! {
                visitor.enter_property(stringify!(#field_name));
                self.#field_name.walk_mut(visitor);
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
                    #visit
                    #fields
                }

                fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
                    #visit_mut
                    #fields_mut
                }
            }
        }
    }
}

/// Derive the `WalkNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let (visit, visit_mut) = match enum_name.to_string().as_str() {
        name @ ("Node" | "CreativeWorkType" | "Block" | "Inline") => {
            let (method, method_mut) = match name {
                "Node" => (quote!(visit_node), quote!(visit_node_mut)),
                "CreativeWorkType" => (quote!(visit_work), quote!(visit_work_mut)),
                "Block" => (quote!(visit_block), quote!(visit_block_mut)),
                "Inline" => (quote!(visit_inline), quote!(visit_inline_mut)),
                _ => unreachable!(),
            };

            (
                quote! {
                    if visitor.#method(self).is_break() {
                        return
                    }
                },
                quote! {
                    if visitor.#method_mut(self).is_break() {
                        return
                    }
                },
            )
        }
        _ => (TokenStream::new(), TokenStream::new()),
    };

    let mut variants = TokenStream::new();
    let mut variants_mut = TokenStream::new();
    for variant in &data.variants {
        if matches!(variant.fields, Fields::Named(..) | Fields::Unnamed(..)) {
            let variant_name = &variant.ident;
            variants.extend(quote! {
                Self::#variant_name(variant) => { variant.walk(visitor); },
            });
            variants_mut.extend(quote! {
                Self::#variant_name(variant) => { variant.walk_mut(visitor); },
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
            }
        }
    }
}
