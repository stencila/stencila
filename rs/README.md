# Stencila Schema for Rust

[![Build Status](https://dev.azure.com/stencila/stencila/_apis/build/status/stencila.schema?branchName=master)](https://dev.azure.com/stencila/stencila/_build/latest?definitionId=9&branchName=master)

This crate provides Rust bindings for the [Stencila Schema](https://schema.stenci.la). It is primarily aimed at Rust developers wanting to programmatically generate or modify documents, particularly executable documents. For example, it is used in the [Stencila Rust library](https://github.com/stencila/stencila/tree/master/rust).

## Install

```sh
cargo add stencila_schema
```

## Use

This package exports a `struct` for each type of document node in the Stencila Schema e.g. `Article`, `Paragraph`, `CodeChunk`.

Note that all node properties e.g. `familyNames` are made snake case e.g. `family_names` for consistency with Rust conventions.

```rust
let article = Article {
    title: Some(ArticleTitle::String("The article title".into())),
    authors: Some(vec![ArticleAuthors::Person({
        Person {
            given_names: Some(vec!["Jane".into()]),
            family_names: Some(vec!["Jones".into()]),
            ..Default::default()
        }
    })]),
    content: Some(vec![BlockContent::Paragraph(Paragraph {
        content: vec![
            InlineContent::String("A paragraph with a ".into()),
            InlineContent::CodeExpression(CodeExpression {
                programming_language: Some("r".into()),
                text: "2^2".into(),
                ..Default::default()
            }),
        ],
        ..Default::default()
    })]),
    ..Default::default()
};
```

## Develop

Get started by cloning this repository and using `cargo` to run the tests,

```bash
git clone git@github.com:stencila/schema
cd schema/rs
cargo test
```

Of course, you need to have Rust [installed](https://rustup.rs). If you want to re-generate `src/types.rs`, you'll also need to have Node.js installed (to generate the Rust code from the schema's YAML files).
