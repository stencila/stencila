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
make format lint test
```

### Code organization

This library is made up of several Rust crates. Splitting Rust code into many small creates has advantages for compilation speed, modularization and reuse. Most of these crates are internal and are not published (those crates have `version = "0.0.0"` in their `Cargo.toml`).

The current crates include:

#### CLI

- [`stencila`](stencila): The `stencila` CLI tool

#### Schema

- [`schema-gen`](schema-gen): Generates language bindings, JSON Schema definitions and documentation from the Stencila Schema.

- [`schema`](schema): Rust types generated from the Stencila Schema by `schema-gen`

#### Codecs

- [`codec`](codec) `🏗️ In progress`: The `Codec` trait for encoding/decoding between the types in the `schema` crate and other external formats (i.e. a 'converter').

- [`codec-json`](codec-json): A `Codec` for [JSON](https://json.org/).

- [`codec-json5`](codec-json5): A `Codec` for [JSON5](https://json5.org/).

- [`codec-yaml`](codec-yaml): A `Codec` for [YAML](https://yaml.org/).

- [`codec-jats`](codec-jats) `🏗️ In progress`: A `Codec` for [JATS XML](https://jats.nlm.nih.gov/).

- [`codec-html`](codec-html) `🏗️ In progress`: A `Codec` for [HTML](https://developer.mozilla.org/en-US/docs/Web/HTML).

#### Utilities

- [`common`](common): Common dependencies used across crates.

- [`common-dev`](common-dev): Common development dependencies used across crates.


### Releases

To create a release do,

```console
cargo release -p stencila --no-publish alpha --execute
```

This will increment the version of the `stencila` package, create a Git tag, and trigger the `release.yaml` Github Actions workflow.
