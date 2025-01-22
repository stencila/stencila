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
    authors: bool,

    #[darling(default)]
    provenance: bool,

    #[darling(default)]
    metadata: bool,

    #[darling(default)]
    content: bool,

    #[darling(default)]
    archive: bool,

    #[darling(default)]
    temporary: bool,

    #[darling(default)]
    code: bool,

    #[darling(default)]
    compilation: bool,

    #[darling(default)]
    execution: bool,

    #[darling(default)]
    output: bool,

    #[darling(default)]
    timestamps: bool,
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
        let Some(field_name) = field.ident else {
            return;
        };
        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        let Type::Path(type_path) = field.ty else {
            return;
        };
        let Some(PathSegment {
            ident: field_type, ..
        }) = type_path.path.segments.last()
        else {
            return;
        };

        // The tokens used to strip the field
        let strip = if field_type == "Option" {
            quote! { = None }
        } else if field_type == "String"
            || field_type == "Cord"
            || field_type == "Vec"
            || field_type == "HashMap"
            || field_type == "IndexMap"
        {
            quote! { .clear() }
        } else {
            quote! { = Default::default() }
        };

        if !strip.is_empty() {
            // Strip the field if it is in targeted scopes
            if field.authors {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Authors) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.provenance {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Provenance) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.metadata {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Metadata) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.content {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Content) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.archive {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Archive) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.temporary {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Temporary) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.code {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Code) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.compilation {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Compilation) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.execution {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Execution) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.output {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Output) {
                        self.#field_name #strip;
                    }
                })
            }

            if field.timestamps {
                fields.extend(quote! {
                    if targets.scopes.contains(&StripScope::Timestamps) {
                        self.#field_name #strip;
                    }
                })
            }

            // Strip field if it is in properties
            fields.extend(quote! {
                if targets.properties.iter().any(|prop|
                    prop.as_str() == stringify!(#field_name) ||
                    prop.as_str() == concat!(stringify!(#struct_name), ".", stringify!(#field_name))
                ) {
                    self.#field_name #strip;
                }
            })
        }

        // Recursively call strip
        fields.extend(quote! {
            self.#field_name.strip(targets);
        })
    });

    quote! {
        impl StripNode for #struct_name {
            fn strip(&mut self, targets: &StripTargets) -> &mut Self {
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
            impl StripNode for #enum_name {}
        }
    } else {
        quote! {
            impl StripNode for #enum_name {
                fn strip(&mut self, targets: &StripTargets) -> &mut Self {
                    match self {
                        #variants
                    }
                    self
                }
            }
        }
    }
}
