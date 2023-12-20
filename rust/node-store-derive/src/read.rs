use common::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields},
};

/// Derive the `ReadNode` trait
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data, &input.attrs),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `ReadNode` trait for a `struct`
pub fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `load_map` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_ident = &field.ident;
        let field_name = &field_ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();

        let field = if field_name == "r#type" || field_name == "uuid" {
            quote! {}
        } else if field_name == "options" {
            quote! {
                let prop = node_store::Prop::Map("options".to_string());
                if crdt.get(obj_id, prop.clone())?.is_some() {
                    node.options.load_from(crdt, &obj_id, prop)?;
                }
            }
        } else {
            quote! {
                node.#field_ident.load_from(crdt, obj_id, stringify!(#field_ident).into())?;
            }
        };

        fields.extend(field);
    }
    methods.extend(quote! {
        fn load_map<C: node_store::ReadCrdt>(crdt: &C, obj_id: &node_store::ObjId) -> common::eyre::Result<Self> {
            // Create a new node
            let mut node = Self::default();

            // Load each field from the CRDT
            #fields

            Ok(node)
        }
    });

    quote! {
        impl node_store::ReadNode for #struct_name {
            #methods
        }
    }
}

/// Derive the `ReadNode` trait for an `enum`
pub fn derive_enum(input: &DeriveInput, data: &DataEnum, attrs: &Vec<Attribute>) -> TokenStream {
    let enum_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `load_map` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                stringify!(#variant_name) => Ok(Self::#variant_name(#variant_name::load_map(crdt, obj_id)?)),
            },
            Fields::Unit => quote! {
                stringify!(#variant_name) => common::eyre::bail!(
                    "Attempting to load unit variant `{}::{}` as map",
                    stringify!(#enum_name),
                    stringify!(#variant_name)
                ),
            },
        };
        cases.extend(case)
    }
    methods.extend(quote! {
        fn load_map<C: node_store::ReadCrdt>(crdt: &C, obj_id: &node_store::ObjId) -> common::eyre::Result<Self> {
            let Some(node_type) = node_store::get_type(crdt, obj_id)? else {
                common::eyre::bail!("Automerge object has no `type` property needed for loading enum `{}`", stringify!(#enum_name));
            };
            match node_type.as_str() {
                #cases
                _ => common::eyre::bail!("Unexpected type `{node_type}` in Automerge store for enum `{}`", stringify!(#enum_name))
            }
        }
    });

    // Derive `load_str` method for enums with all unit variants
    if data
        .variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit))
    {
        methods.extend(quote! {
            fn load_str(value: &common::smol_str::SmolStr) -> common::eyre::Result<Self> {
                Ok(serde_json::from_str(&["\"", &value, "\""].concat())?)
            }
        });
    }

    // Derive `load_none` for enums that implement `Defaults` trait
    // This is like using the `#serde[default]` attr.
    // It seems that we are unable to check the `derive` attr, maybe because
    // we are "in it" at this point in the code.
    let mut impl_default = false;
    for attr in attrs {
        if attr.path().is_ident("def") {
            impl_default = true;
        }
    }
    if impl_default {
        methods.extend(quote! {
            fn load_none() -> common::eyre::Result<Self> {
                Ok(Self::default())
            }
        });
    }

    quote! {
        impl node_store::ReadNode for #enum_name {
            #methods
        }
    }
}
