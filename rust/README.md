# Stencila Rust

**Core Stencila functionality implemented in a fast, memory-safe language**

## 🦀 Introduction

This is the `stencila` core Rust library. Its main purpose is to implement core functionality that can be reused in the Stencila CLI and in language bindings.

## 📦 Install

This library is not yet published to https://crates.io/, but you can still add it to your `Cargo.toml` using,

```toml
stencila = { git = "https://github.com/stencila/stencila" }
```

## 🛠️ Develop

### Getting started

Get started by cloning this repository, [installing Rust](https://rustup.rs) and using the using `make setup` to install necessary Cargo plugins (for formatting, linting, etc):

```sh
git clone git@github.com:stencila/stencila
cd stencila/rust
make setup
```

If you are contributing code please run formatting, linting and tests before submitting PRs:

```sh
make fix test
```

### Code organization

This library is made up of several Rust crates. Splitting Rust code into many small creates has advantages for compilation speed, modularization and reuse. Most of these crates are internal and are not published (those crates have `version = "0.0.0"` in their `Cargo.toml`).

The current crates include:

#### CLI

- [`stencila`](stencila): The `stencila` CLI tool

#### Documents and formats

- [`document`](document): A document representing one of the `CreativeWork` types in the Stencila Schema.

- [`format`](format): Provides the `Format` enum and utility functions for working with document formats.

#### Schema

- [`schema-gen`](schema-gen): Generates language bindings, JSON Schema definitions and documentation from the Stencila Schema.

- [`schema`](schema): Rust types generated from the Stencila Schema by `schema-gen`

#### Node traits

- [`node-html`](node-html) and [`node-html-derive`](node-html-derive): Provides the `ToHtml` trait and macro for generating HTML for document nodes.

- [`node-store`](node-store) and [`node-store-derive`](node-store-derive): Provides the `Read` and `Write` traits and macros for reading and writing document nodes from/to Automerge stores.

- [`node-strip`](node-strip) and [`node-strip-derive`](node-strip-derive): Provides the `Strip` trait and macro for removing properties of document nodes.

#### Codecs

- [`format`](format): Provides the `Format` enum which describes alternative document formats.

- [`codec`](codec): Provies the `Codec` trait for encoding/decoding between the types in the `schema` crate and other external formats (i.e. a 'converter').

- [`codec-debug`](codec-debug): A codec for the Rust [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html) trait.

- [`codec-html`](codec-html) `🏗️ In progress`: A codec for [HTML](https://developer.mozilla.org/en-US/docs/Web/HTML).

- [`codec-jats`](codec-jats) `🏗️ In progress`: A codec for [JATS XML](https://jats.nlm.nih.gov/).

- [`codec-json`](codec-json): A codec for [JSON](https://json.org/).

- [`codec-json5`](codec-json5): A codec for [JSON5](https://json5.org/).

- [`codec-ron`](codec-ron): A codec for [RON](https://github.com/ron-rs/ron).

- [`codec-text`](codec-text): A codec for plain text.

- [`codec-yaml`](codec-yaml): A codec for [YAML](https://yaml.org/).

#### Utilities

- [`common`](common): Common dependencies used across crates.

- [`common-dev`](common-dev): Common development dependencies used across crates.

### Tokio console

Turn on the `console-subscriber` feature to use [`tokio-console`](https://github.com/tokio-rs/console) for debugging async tasks and locks e.g.

```console
cargo run --bin stencila --features=console-subscriber -- serve
```

and in another terminal run `tokio-console`,

```console
tokio-console
```

### Releases

To create a release do,

```console
cargo release -p stencila --tag-prefix '' --no-publish alpha --execute
```

This will increment the version of the `stencila` package, create a Git tag, and trigger the `release.yaml` Github Actions workflow. You'll also have to mark the release as 'Latest' on Github (if it is not already) for it to be used by default by the `install.sh` script in the root of this repo.
