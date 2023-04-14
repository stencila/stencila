use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive the `Read` trait
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = match &input.data {
        Data::Struct(data) => derive_struct(&input, data),
        Data::Enum(data) => derive_enum(&input, data),
        Data::Union(..) => return proc_macro::TokenStream::new(),
    };

    proc_macro::TokenStream::from(tokens)
}

/// Derive the `Read` trait for a `struct`
pub fn derive_struct(input: &DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let struct_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `load_map` method
    let mut fields = TokenStream::new();
    for field in &data.fields {
        let field_name = &field.ident;
        let field_name_string = &field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let field = if field_name_string == "r#type" {
            quote! {}
        } else if field_name_string == "id" {
            quote! {
                node.id = Some(node_store::id_to_base64(obj_id));
            }
        } else {
            quote! {
                node.#field_name.load_from(store, obj_id, stringify!(#field_name).into())?;
            }
        };
        fields.extend(field);
    }
    methods.extend(quote! {
        fn load_map<S: node_store::ReadStore>(store: &S, obj_id: &node_store::ObjId) -> common::eyre::Result<Self> {
            // Create a new node
            let mut node = Self::default();

            // Load each field from the store
            #fields

            Ok(node)
        }
    });

    quote! {
        impl node_store::Read for #struct_name {
            #methods
        }
    }
}

/// Derive the `Read` trait for an `enum`
pub fn derive_enum(input: &DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut methods = TokenStream::new();

    // Derive `load_map` method
    let mut cases = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let case = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                stringify!(#variant_name) => Ok(Self::#variant_name(#variant_name::load_map(store, obj_id)?)),
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
        fn load_map<S: node_store::ReadStore>(store: &S, obj_id: &node_store::ObjId) -> common::eyre::Result<Self> {
            let r#type = node_store::get_type::<Self,_>(store, obj_id)?;
            match r#type.as_str() {
                #cases
                _ => common::eyre::bail!("Unexpected type `{}` in Automerge store for enum `{}`", r#type, stringify!(#enum_name))
            }
        }
    });

    quote! {
        impl node_store::Read for #enum_name {
            #methods
        }
    }
}
