//! Provides the `LatexCodec` derive macro for structs and enums in Stencila Schema

use darling::{self, FromDeriveInput, FromField};

use common::{
    inflector::Inflector,
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident},
};

#[derive(FromDeriveInput)]
#[darling(attributes(latex))]
struct TypeAttr {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, FieldAttr>,

    #[darling(default)]
    command: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(latex))]
struct FieldAttr {
    ident: Option<Ident>,
}

/// Derive the `LatexCodec` trait for a `struct` or an `enum`
#[proc_macro_derive(LatexCodec, attributes(latex))]
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

/// Derive the `LatexCodec` trait for a `struct`
fn derive_struct(type_attr: TypeAttr) -> TokenStream {
    let struct_name = type_attr.ident;

    if struct_name.to_string().ends_with("Options") {
        return TokenStream::new();
    }

    let (command_enter, command_exit) = if let Some(command) = type_attr.command {
        (
            quote!(context.command_enter(#command);),
            quote!(context.command_exit();),
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    let mut fields = TokenStream::new();
    type_attr.data.map_struct_fields(|field_attr| {
        let Some(field_name) = field_attr.ident else {
            return
        };

        // Only encode content
        if !(field_name == "content" || field_name == "value"  || field_name == "code") {
            return;
        }

        let property = Ident::new(&field_name.to_string().to_pascal_case(), Span::call_site());

        let field_tokens = quote! {
            context.property_fn(NodeProperty::#property, |context| self.#field_name.to_latex(context));
        };

        fields.extend(field_tokens)
    });

    quote! {
        impl LatexCodec for #struct_name {
            fn to_latex(&self, context: &mut LatexEncodeContext) {
                context.enter_node(self.node_type(), self.node_id());
                #command_enter
                #fields
                #command_exit
                context.exit_node();
            }
        }
    }
}

/// Derive the `LatexCodec` trait for an `enum`
fn derive_enum(type_attr: TypeAttr, data: &DataEnum) -> TokenStream {
    let enum_name = type_attr.ident;

    let mut variants = TokenStream::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_tokens = match &variant.fields {
            Fields::Named(..) | Fields::Unnamed(..) => quote! {
                Self::#variant_name(variant) => { variant.to_latex(context); },
            },
            Fields::Unit => quote! {
                Self::#variant_name => { context.str(stringify!(#variant_name)); },
            },
        };
        variants.extend(variant_tokens)
    }

    quote! {
        impl LatexCodec for #enum_name {
            fn to_latex(&self, context: &mut LatexEncodeContext) {
                match self {
                    #variants
                }
            }
        }
    }
}
