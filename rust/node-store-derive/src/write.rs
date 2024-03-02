use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields},
};

/// Derive the `WriteNode` trait
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `WriteNode` trait for a `struct`
pub fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `sync_map` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_ident = &field.ident;
        let field_name = &field_ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();

        let field = if field_name == "r#type" {
            // Always put the `type` to the Automerge map
            quote! {
                store.put::<_,_,&str>(obj_id, "type", stringify!(#struct_name))?;
            }
        } else if field_name == "uid" {
            // Never put the uid into the Automerge map (because we use the ObjId on load)
            continue;
        } else if field_name == "options" {
            // Sync options into the Automerge map (i.e. "flatten" them)
            quote! {
                self.options.sync_map(store, obj_id)?;
            }
        } else {
            // Put other fields into the Automerge map as props
            quote! {
                self.#field_ident.put_prop(store, obj_id, stringify!(#field_ident).into())?;
            }
        };
        fields.extend(field);
    }
    methods.extend(quote! {
        fn sync_map(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId) -> common::eyre::Result<()> {
            use node_store::automerge::{ReadDoc, transaction::Transactable};

            #fields

            Ok(())
        }
    });

    // Derive `insert_prop` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_ident = &field.ident;
        let field_name = &field_ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();

        let field = if field_name == "r#type" {
            // Always put the `type` to the Automerge map
            quote! {
                store.put::<_,_,&str>(&obj_id, "type", stringify!(#struct_name))?;
            }
        } else if field_name == "uid" {
            // Never put the uid in the Automerge map (because we use the obj_id on load)
            continue;
        } else if field_name == "options" {
            // Insert options into the Automerge map (i.e. "flatten" them)
            quote! {
                self.options.insert_into(store, obj_id)?;
            }
        } else {
            // Insert other fields into the Automerge map as properties
            quote! {
                self.#field_ident.insert_prop(store, &obj_id, stringify!(#field_ident).into())?;
            }
        };
        fields.extend(field);
    }
    methods.extend(quote! {
        fn insert_prop(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId, prop: node_store::Prop) -> common::eyre::Result<()> {
            use node_store::{ObjType, Prop, automerge::{transaction::Transactable}};

            let prop_obj_id = match prop.clone() {
                Prop::Map(key) => store.put_object(obj_id, key, ObjType::Map)?,
                Prop::Seq(index) => store.insert_object(obj_id, index, ObjType::Map)?,
            };

            self.insert_into(store, &prop_obj_id)?;

            Ok(())
        }

        fn insert_into(&self, store: &mut node_store::WriteStore, obj_id: &node_store::ObjId) -> common::eyre::Result<()> {
            use node_store::automerge::transaction::Transactable;

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
        impl node_store::WriteNode for #struct_name {
            #methods
        }
    }
}

/// Derive the `WriteNode` trait for an `enum`
pub fn derive_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `sync_map` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => variant.sync_map(store, obj_id),
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
                Self::#variant_name(variant) => variant.insert_prop(store, obj_id, prop),
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
                Self::#variant_name(variant) => variant.put_prop(store, obj_id, prop),
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
        impl node_store::WriteNode for #enum_name {
            #methods
        }
    }
}
