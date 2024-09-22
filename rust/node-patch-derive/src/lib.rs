//! Provides the `PatchNode` derive macro for structs and enums in Stencila Schema

use darling::{self, ast::Data as AstData, util::Ignored, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{parse_macro_input, parse_str, Data, DataEnum, DeriveInput, Fields, Ident, Path},
};

#[derive(FromDeriveInput)]
#[darling(attributes(patch))]
struct TypeAttr {
    ident: Ident,
    data: AstData<Ignored, FieldAttr>,

    authors_on: Option<String>,

    #[darling(default)]
    authors_take: bool,

    apply_with: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(patch))]
struct FieldAttr {
    ident: Option<Ident>,

    #[darling(multiple, rename = "format")]
    formats: Vec<String>,
}

/// Derive the `PatchNode` trait for a `struct` or an `enum`
#[proc_macro_derive(PatchNode, attributes(patch))]
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

    let is_options = struct_name.to_string().ends_with("Options");
    let mut has_options = false;
    let mut has_provenance = false;

    let mut authorship_fields = TokenStream::new();
    let mut provenance_fields = TokenStream::new();
    let mut similarity_fields = TokenStream::new();
    let mut diff_fields = TokenStream::new();
    let mut patch_fields = TokenStream::new();
    let mut apply_fields = TokenStream::new();
    let mut apply_verify_fields = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return;
        };
        if field_name == "r#type" || field_name == "uid" {
            return;
        }

        // Authorship, similarity, and diffing are not delegated down to fields in options,
        // but the `apply` method does. Record if has an `options` field so we can
        // delegate `apply` to that for properties not on here.
        if field_name == "options" {
            has_options = true;
            return;
        }

        // Authors field is not patched
        if field_name == "authors" {
            return;
        }

        // Provenance should not need to be patched etc
        if field_name == "provenance" {
            has_provenance = true;
            return;
        }

        let property = Ident::new(
            &field_name.to_string().replace("r#", "").to_pascal_case(),
            Span::call_site(),
        );

        // Do no apply authorship and provenance to any `authors` field (e.g. on SoftwareApplication)
        // and other common, atomic properties
        if field_name != "id" && field_name != "authors" {
            authorship_fields.extend(quote! {
                self.#field_name.authorship(context)?;
            });
            provenance_fields.extend(quote! {
                self.#field_name.provenance(),
            });
        }

        // Application of patches is implemented for all fields
        patch_fields.extend(quote! {
            context.enter_property(NodeProperty::#property);
            if self.#field_name.patch(patch, context)? {
                return Ok(true);
            }
            context.exit_property();
        });
        apply_fields.extend(quote! {
            NodeProperty::#property => {
                self.#field_name.apply(path, op, context)?;
            },
        });
        apply_verify_fields.extend(quote! {
            self.#field_name.apply(path, op.clone(), context)?;
        });

        // Diffing related methods are conditionally implemented based on the format
        let field_diffed = if field_attr.formats.contains(&"all".to_string()) {
            // Field should be diffed for all formats
            quote! { true }
        } else {
            // Field should be diffed if the context has no, or lossless format, or if
            // the context format is explicitly listed.
            let mut condition = quote! {
                context.format.is_none() || context.format.as_ref().map(|format| format.is_lossless()).unwrap_or_default()
            };
            if field_attr.formats.contains(&"md".to_string()) {
                condition.extend(quote! {
                    || matches!(context.format, Some(Format::Markdown))
                })
            }
            if field_attr.formats.contains(&"myst".to_string()) {
                condition.extend(quote! {
                    || matches!(context.format, Some(Format::Myst))
                })
            }
            condition
        };

        similarity_fields.extend(quote! {
            if #field_diffed {
                fields.push(self.#field_name.similarity(&other.#field_name, context)?);
            }
        });

        diff_fields.extend(quote! {
            if #field_diffed {
                context.enter_property(NodeProperty::#property);
                self.#field_name.diff(&other.#field_name, context)?;
                context.exit_property();
            }
        });
    });

    let call_update_and_release_authors = |overwrite: bool| {
        if let Some(authors_on) = &type_attr.authors_on {
            let authors = if authors_on == "options" {
                quote! { self.options.authors }
            } else {
                quote! { self.authors }
            };

            let take = type_attr.authors_take;

            (
                quote! {
                    let authors_taken = context.update_authors(&mut #authors, #take, #overwrite);
                },
                quote! {
                    if authors_taken { context.release_authors() };
                },
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        }
    };

    let call_update_provenance = if has_provenance && !provenance_fields.is_empty() {
        quote! {
            PatchContext::update_provenance(&mut self.provenance, vec![
                #provenance_fields
            ]);
        }
    } else {
        TokenStream::new()
    };

    let authors = if let Some(authors_on) = &type_attr.authors_on {
        let authors = if authors_on == "options" {
            quote! { self.options.authors }
        } else {
            quote! { self.authors }
        };

        quote! {
            fn authors(&self) -> Option<Vec<Author>> {
                #authors.clone()
            }
        }
    } else {
        TokenStream::new()
    };

    let authorship = if !authorship_fields.is_empty() {
        let (call_update_authors, call_release_authors) = call_update_and_release_authors(true);
        quote! {
            fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
                #call_update_authors

                #authorship_fields

                #call_release_authors
                #call_update_provenance

                Ok(())
            }
        }
    } else {
        TokenStream::new()
    };

    let provenance = if has_provenance {
        // The struct has provence so pass that up
        quote! {
            fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
                self.provenance.clone()
            }
        }
    } else if !provenance_fields.is_empty() {
        // The struct has fields that potentially have provenance
        // so flatten those and pass up
        quote! {
            fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
                PatchContext::flatten_provenance(vec![
                    #provenance_fields
                ])
            }
        }
    } else {
        TokenStream::new()
    };

    let similarity = if !similarity_fields.is_empty() {
        quote! {
            fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
                let mut fields = Vec::new();
                #similarity_fields
                PatchContext::mean_similarity(fields)
            }
        }
    } else {
        TokenStream::new()
    };

    let diff = if !diff_fields.is_empty() {
        quote! {
            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                #diff_fields
                Ok(())
            }
        }
    } else {
        TokenStream::new()
    };

    let patch = if !is_options {
        // If no fields applied patch and has option, then fallback to trying that
        let end = if has_options {
            quote! { self.options.patch(patch, context) }
        } else {
            quote! { Ok(false) }
        };

        quote! {
            fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
                if let Some(node_id) = patch.node_id.as_ref() {
                    if node_id == &self.node_id() {
                        return patch.apply(self, context);
                    }
                } else {
                    return patch.apply(self, context);
                }

                #patch_fields

                #end
            }
        }
    } else {
        // For options, there is no node_id, so just attempt to apply patch to fields
        quote! {
            fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
                #patch_fields

                Ok(false)
            }
        }
    };

    let apply = {
        let call_apply_with = match &type_attr.apply_with {
            Some(apply_with) => {
                let func =
                    parse_str::<Path>(apply_with).expect("invalid patch `apply_with` option");
                quote! {
                    if #func(self, path, &op, context)? {
                        return Ok(())
                    }
                }
            }
            None => TokenStream::new(),
        };

        let (call_update_authors, call_release_authors) = call_update_and_release_authors(false);

        let unmatched_field = if has_options {
            // Put the property back on to the path and try in options
            quote! {
                {
                    path.push_back(PatchSlot::Property(property));
                    self.options.apply(path, op, context)?;
                }
            }
        } else {
            quote! {
                bail!("Invalid property `{property}` for struct `{}`", stringify!(#struct_name))
            }
        };

        quote! {
            fn apply(&mut self, path: &mut PatchPath, op: PatchOp, context: &mut PatchContext) -> Result<()> {
                #call_apply_with

                #call_update_authors

                if matches!(op, PatchOp::Verify) {
                    #apply_verify_fields;
                } else if !matches!(op, PatchOp::Nothing) {
                    let Some(PatchSlot::Property(property)) = path.pop_front() else {
                        bail!("Invalid empty patch path for `{}`", stringify!(#struct_name));
                    };

                    match (property) {
                        #apply_fields
                        _ => #unmatched_field
                    }
                }

                #call_release_authors
                #call_update_provenance

                Ok(())
            }
        }
    };

    quote! {
        impl PatchNode for #struct_name {
            #authors
            #authorship
            #provenance
            #similarity
            #diff
            #patch
            #apply
        }
    }
}

/// Derive the `PatchNode` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut is_unit = false;
    let mut authors_variants = TokenStream::new();
    let mut authorship_variants = TokenStream::new();
    let mut provenance_variants = TokenStream::new();
    let mut similarity_variants = TokenStream::new();
    let mut diff_variants = TokenStream::new();
    let mut patch_variants = TokenStream::new();
    let mut apply_variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => {
                authors_variants.extend(quote! {
                    Self::#variant_name(me) => me.authors(),
                });
                authorship_variants.extend(quote! {
                    Self::#variant_name(me) => me.authorship(context),
                });
                provenance_variants.extend(quote! {
                    Self::#variant_name(me) => me.provenance(),
                });
                similarity_variants.extend(quote! {
                    (Self::#variant_name(me), Self::#variant_name(other)) => me.similarity(other, context),
                });
                diff_variants.extend(quote! {
                    (Self::#variant_name(me), Self::#variant_name(other)) => me.diff(other, context),
                });
                patch_variants.extend(quote! {
                    Self::#variant_name(me) => me.patch(patch, context),
                });
                apply_variants.extend(quote! {
                    Self::#variant_name(me) => me.apply(path, op, context),
                });
            }
            Fields::Unit => {
                is_unit = true;
                authors_variants.extend(quote! {
                    Self::#variant_name => None,
                });
                authorship_variants.extend(quote! {
                    Self::#variant_name => Ok(()),
                });
                provenance_variants.extend(quote! {
                    Self::#variant_name => None,
                });
                similarity_variants.extend(quote! {
                    (Self::#variant_name, Self::#variant_name) => Ok(1.0),
                });
                diff_variants.extend(quote! {
                    (Self::#variant_name, Self::#variant_name) => Ok(()),
                });
            }
        };
    }

    let (patch, apply) = if is_unit {
        (
            quote! {
                Ok(false)
            },
            quote! {
                if matches!(op, PatchOp::Verify) {
                    Ok(())
                } else if let PatchOp::Set(value) = op {
                    *self = Self::from_value(value)?;
                    Ok(())
                } else {
                    bail!("Invalid op for enum `{}`", stringify!(#enum_name));
                }
            },
        )
    } else {
        (
            quote! {
                match self {
                    #patch_variants
                }
            },
            quote! {
                match self {
                    #apply_variants
                }
            },
        )
    };

    let (to_value, from_value) = match enum_name.to_string().as_str() {
        "Inline" | "Block" | "Node" => (
            quote! {
                Ok(PatchValue::#enum_name(self.clone()))
            },
            quote! {
                match value {
                    PatchValue::#enum_name(value) => Ok(value),
                    PatchValue::Json(value) => Ok(serde_json::from_value(value)?),
                    _ => bail!("Invalid value for `{}`", stringify!(#enum_name))
                }
            },
        ),
        _ => {
            if is_unit {
                (
                    quote! {
                        Ok(PatchValue::String(self.to_string()))
                    },
                    quote! {
                        match value {
                            PatchValue::Json(json) => Ok(serde_json::from_value(json)?),
                            PatchValue::String(string) => Ok(string.parse()?),
                            _ => bail!("Invalid patch value for unit enum `{}`", stringify!(#enum_name))
                        }
                    },
                )
            } else {
                (
                    quote! {
                        Ok(PatchValue::Json(serde_json::to_value(self)?))
                    },
                    quote! {
                        match value {
                            PatchValue::Json(json) => Ok(serde_json::from_value(json)?),
                            _ => bail!("Invalid patch value for enum `{}`", stringify!(#enum_name))
                        }
                    },
                )
            }
        }
    };

    quote! {
        impl PatchNode for #enum_name {
            fn authors(&self) -> Option<Vec<Author>> {
                match self {
                    #authors_variants
                }
            }

            fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
                match self {
                    #authorship_variants
                }
            }

            fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
                match self {
                    #provenance_variants
                }
            }

            fn to_value(&self) -> Result<PatchValue> {
                #to_value
            }

            fn from_value(value: PatchValue) -> Result<Self> {
                #from_value
            }

            fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
                match (self, other) {
                    // Same variants
                    #similarity_variants
                    // Different variants: zero similarity
                    _ => Ok(0.0)
                }
            }

            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                match (self, other) {
                    // Same variants
                    #diff_variants
                    // Different variants: set with other
                    _ => {
                        context.op_set(other.to_value()?);
                        Ok(())
                    }
                }
            }

            fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
                #patch
            }

            fn apply(&mut self, path: &mut PatchPath, op: PatchOp, context: &mut PatchContext) -> Result<()> {
                #apply
            }
        }
    }
}
