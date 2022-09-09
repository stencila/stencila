//! Generate `config::Docs` trait for `struct`s and `enum`s

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    {parse_macro_input, Attribute, DeriveInput, FieldsNamed, Ident, Variant},
};

use common::{itertools::Itertools, once_cell::sync::Lazy, regex::Regex};

#[proc_macro_derive(Docs, attributes(serde))]
pub fn derive_signature(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = ast.data
    {
        docs_for_struct(ast.ident, &ast.attrs, fields)
    } else if let syn::Data::Enum(syn::DataEnum { variants, .. }) = ast.data {
        docs_for_enum(ast.ident, &ast.attrs, variants)
    } else {
        quote! {}.into()
    }
}

fn docs_for_struct(name: Ident, attrs: &[Attribute], fields: FieldsNamed) -> TokenStream {
    let docs = docs_from_attrs(attrs);

    let mut names = Vec::new();
    let mut idents = Vec::new();
    let mut types = Vec::new();
    let mut descs = Vec::new();
    for field in fields.named.iter() {
        idents.push(&field.ident);

        let field_ident: &syn::Ident = field.ident.as_ref().unwrap();
        let field_name: String = field_ident.to_string();
        let field_lit_str = syn::LitStr::new(&field_name, field.span());
        names.push(quote! { #field_lit_str });

        types.push(field.ty.to_token_stream());

        descs.push(docs_from_attrs(&field.attrs));
    }

    quote! {
        impl Docs for #name {
            fn docs(&self) -> String {
                use std::fmt::Write;

                let mut docs = format!("# {} `struct`\n\n", stringify!(#name));
                writeln!(&mut docs, "{}\n\n", #docs.trim()).ok();

                #(
                    writeln!(
                        &mut docs,
                        "## `{name}`: `{type_name}`\n\n{docs}\n\n",
                        name = #names,
                        type_name = stringify!(#types).replace(" ", ""),
                        docs = #descs
                    ).ok();
                )*

                docs += "\n\n";

                #(
                    docs += &self.#idents.docs();
                )*

                docs
            }
        }
    }
    .into()
}

fn docs_for_enum(
    enum_name: Ident,
    attrs: &[Attribute],
    variants: Punctuated<Variant, Comma>,
) -> TokenStream {
    let docs = docs_from_attrs(attrs);

    let variants = variants
        .iter()
        .map(|variant| {
            format!(
                "| `{name}` | {desc} |\n",
                name = variant.ident,
                desc = docs_from_attrs(&variant.attrs).trim()
            )
        })
        .collect_vec();

    quote! {
        impl Docs for #enum_name {
            fn docs(&self) -> String {
                let mut docs = format!("# {} `enum`\n\n", stringify!(#enum_name));
                docs += #docs.trim();
                docs += "\n\n";

                docs += "| Variant | Description |\n";
                docs += "| ------- | ----------- |\n";
                #(
                    docs += &#variants;
                )*

                docs + "\n\n"
            }
        }
    }
    .into()
}

fn docs_from_attrs(attrs: &[syn::Attribute]) -> String {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"^\s*=\s*"\s?(?P<content>.*?)"\s*$"#).unwrap());

    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("doc"))
        .fold(String::new(), |mut docs, attr| {
            use std::fmt::Write;

            let doc = TokenStream::from(attr.tokens.clone()).to_string();
            let doc = REGEX.replace_all(&doc, "$content");
            writeln!(&mut docs, "{}", doc).ok();
            docs.replace("\\\"", "\"")
        })
}
