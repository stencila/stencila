//! Provides a `HtmlCodec` derive macro for structs and enums in Stencila Schema

use std::collections::HashMap;

use darling::{self, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
#[darling(attributes(html))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    elem: Option<String>,

    #[darling(default)]
    attribs: HashMap<String, String>,

    #[darling(default)]
    custom: bool,

    #[darling(default)]
    special: bool,
}

#[derive(FromField)]
#[darling(attributes(html))]
struct FieldAttr {
    ident: Option<Ident>,

    #[darling(default)]
    attr: Option<String>,

    #[darling(default)]
    content: bool,

    #[darling(default)]
    slot: Option<String>,

    #[darling(default)]
    flatten: bool,
}

/// Derive the `HtmlCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(HtmlCodec, attributes(html))]
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

/// Derive the `HtmlCodec` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    if type_attr.special {
        return quote! {
            impl codec_html_trait::HtmlCodec for #struct_name {
                fn to_html(&self) -> String {
                    self.to_html_special()
                }

                fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
                    unreachable!()
                }

                fn to_html_attr(&self) -> String {
                    unreachable!()
                }
            }
        };
    }

    let custom_elem = ["stencila-", &struct_name.to_string().to_kebab_case()].concat();

    let elem = type_attr
        .elem
        .clone()
        .unwrap_or_else(|| custom_elem.clone());

    let mut attrs = TokenStream::new();

    for (name, value) in type_attr.attribs.iter().sorted() {
        let name = name.replace("__", "-").replace('_', ":");
        attrs.extend(quote! {
            (#name.to_string(), #value.to_string()),
        })
    }

    if type_attr.custom {
        attrs.extend(quote!(attr("is", #custom_elem),))
    };

    let mut fields = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return
        };

        if field_name == "r#type" {
            // Skip the type field
            return;
        }

        let field_tokens = if field_attr.flatten {
            // Flatten out the attributes and children of the options field
            quote! {
                let mut parts = self.#field_name.to_html_parts();
                attrs.append(&mut parts.1);
                children.append(&mut parts.2);
            }
        } else if field_name == "content" || field_attr.content {
            // Always add content as direct children
            quote! {
                children.push(self.#field_name.to_html());
            }
        } else if let Some(slot) = field_attr.slot {
            // Wrap the field in a slot
            quote! {
                let slot_html = self.#field_name.to_html();
                if !slot_html.is_empty() {
                    children.push(elem(
                        #slot,
                        &[attr("slot", stringify!(#field_name))],
                        &[slot_html]
                    ));
                }
            }
        } else {
            let attr_name = if let Some(attr_name) = field_attr.attr {
                // If `attr` defined then use that as the attr name
                attr_name
            } else if type_attr.elem.is_some() && !type_attr.custom {
                // If not a custom element, prefix property name with data-
                ["data-", &field_name.to_string()].concat()
            } else {
                // Use the property name as the attribute name
                field_name.to_string()
            };

            quote! {
                attrs.push(attr(stringify!(#attr_name), &self.#field_name.to_html_attr()));
            }
        };
        fields.extend(field_tokens)
    });

    quote! {
        impl codec_html_trait::HtmlCodec for #struct_name {
            fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
                use codec_html_trait::encode::{attr, elem};

                let mut attrs = vec![#attrs];
                let mut children = Vec::new();

                #fields

                (#elem, attrs, children)
            }

            fn to_html_attr(&self) -> String {
                serde_json::to_string(self).unwrap_or_default()
            }
        }
    }
}

/// Derive the `HtmlCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants_to_html = TokenStream::new();
    let mut variants_to_parts = TokenStream::new();
    let mut variants_to_attr = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => {
                variants_to_html.extend(quote! {
                    Self::#variant_name(v) => v.to_html(),
                });
                variants_to_parts.extend(quote! {
                    Self::#variant_name(v) => v.to_html_parts(),
                });
                variants_to_attr.extend(quote! {
                    Self::#variant_name(v) => v.to_html_attr(),
                });
            }
            Fields::Unit => {
                variants_to_html.extend(quote! {
                    Self::#variant_name => stringify!(#variant_name).to_string(),
                });
                variants_to_parts.extend(quote! {
                    Self::#variant_name => ("span", vec![], vec![stringify!(#variant_name).to_string()]),
                });
                variants_to_attr.extend(quote! {
                    Self::#variant_name => serde_json::to_string(stringify!(#variant_name)).unwrap_or_default(),
                });
            }
        };
    }

    quote! {
        impl codec_html_trait::HtmlCodec for #enum_name {
            fn to_html(&self) -> String {
                match self {
                    #variants_to_html
                }
            }

            fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
                match self {
                    #variants_to_parts
                }
            }

            fn to_html_attr(&self) -> String {
                match self {
                    #variants_to_attr
                }
            }
        }
    }
}
