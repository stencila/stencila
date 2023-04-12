use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive the `Write` trait
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `Write` trait for a `struct`
///
/// The implementation of methods is largely based on those for `IndexMap`.
pub fn derive_struct(input: &DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `sync_map` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_name = &field.ident;
        let field_name_string = &field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let field = if field_name_string == "r#type" {
            // Always put the `type` to the store
            quote! {
                store.put::<_,_,&str>(obj_id, "type", stringify!(#struct_name))?;
                keys.remove("type");
            }
        } else {
            // Put fields that are in both map and store
            quote! {
                let field_name = stringify!(#field_name);
                self.#field_name.put_prop(store, obj_id, field_name.into())?;
                keys.remove(field_name);
            }
        };
        fields.extend(field);
    }
    methods.extend(quote! {
        fn sync_map(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId) -> common::eyre::Result<()> {
            use node_store::automerge::{ReadDoc, transaction::Transactable};

            // Get the keys of the store map
            let mut keys: std::collections::HashSet<String> = store.keys(obj_id).collect();

            // Put fields into the store map
            #fields

            // Remove keys that are in the store map but not in the struct
            for key in keys {
                store.delete(obj_id, key.as_str())?;
            }

            Ok(())
        }
    });

    // Derive `insert_prop` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_name = &field.ident;
        let field_name_string = &field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let field = if field_name_string == "r#type" {
            quote! {
                store.put::<_,_,&str>(&prop_obj_id, "type", stringify!(#struct_name))?;
            }
        } else {
            quote! {
                self.#field_name.insert_prop(store, &prop_obj_id, stringify!(#field_name).into())?;
            }
        };
        fields.extend(field);
    }
    methods.extend(quote! {
        fn insert_prop(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId, prop: node_store::Prop) -> common::eyre::Result<()> {
            use node_store::{ReadStore, ObjType, Prop, automerge::{transaction::Transactable}};

            // Create the new map in the store
            let prop_obj_id = match prop {
                Prop::Map(key) => store.put_object(obj_id, key, ObjType::Map)?,
                Prop::Seq(index) => store.insert_object(obj_id, index, ObjType::Map)?,
            };

            // Insert fields into the new map
            #fields

            Ok(())
        }
    });

    // Derive `put_prop` method
    // Note that currently this could be made into a function
    // to avoid code bloat
    methods.extend(quote! {
        fn put_prop(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId, prop: node_store::Prop) -> common::eyre::Result<()> {
            use node_store::{ReadStore, ObjType, automerge::{Value, transaction::Transactable}};

            // Get the existing object at the property
            let existing = store.get(obj_id, prop.clone())?;

            if let Some((Value::Object(ObjType::Map), prop_obj_id)) = existing {
                // Existing object is a map so sync it
                self.sync_map(store, &prop_obj_id)
            } else {
                // Remove any existing object of different type
                if existing.is_some() {
                    store.delete(obj_id, prop.clone())?;
                }

                // Insert a new map object
                self.insert_prop(store, obj_id, prop)?;

                Ok(())
            }
        }
    });

    quote! {
        impl node_store::Write for #struct_name {
            #methods
        }
    }
}

/// Derive the `Write` trait for an `enum`
pub fn derive_enum(input: &DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `sync_map` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.sync_map(store, obj_id),
            },
            Fields::Unit => quote! {
                Self::#variant_name => common::eyre::bail!(
                    "Attempting to dump unit variant `{}::{}` as an Automerge object",
                    stringify!(#enum_name),
                    stringify!(#variant_name)
                ),
            },
        };
        cases.extend(case)
    }
    methods.extend(quote! {
        fn sync_map(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId) -> common::eyre::Result<()> {
            match self {
                #cases
            }
        }
    });

    // Derive `insert_prop` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.insert_prop(store, obj_id, prop),
            },
            Fields::Unit => quote! {
                Self::#variant_name => stringify!(#variant_name).to_string().insert_prop(store, obj_id, prop),
            },
        };
        cases.extend(case)
    }
    methods.extend(quote! {
        fn insert_prop(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId, prop: node_store::Prop) -> common::eyre::Result<()> {
            match self {
                #cases
            }
        }
    });

    // Derive `put_prop` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(v) => v.put_prop(store, obj_id, prop),
            },
            Fields::Unit => quote! {
                Self::#variant_name => stringify!(#variant_name).to_string().put_prop(store, obj_id, prop),
            },
        };
        cases.extend(case)
    }
    methods.extend(quote! {
        fn put_prop(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId, prop: node_store::Prop) -> common::eyre::Result<()> {
            match self {
                #cases
            }
        }
    });

    quote! {
        impl node_store::Write for #enum_name {
            #methods
        }
    }
}
