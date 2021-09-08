# Stencila Rust

**Core Stencila functionality implemented in a fast, memory-safe language**

## 🦀 Introduction

This is the `stencila` Rust library. Its main purpose is to implement core functionality that can be reused in the Stencila [CLI](../cli) and [Desktop](../desktop). There are also bindings to this library for other languages:

- [Node.js](../node) (upon which the Desktop is built)
- [Python](../python) (_experimental_)
- [R](../r) (_experimental_)

## 📦 Install

You'll need to have [Rust installed](https://rustup.rs) first.

This library is not yet published as a Rust crate, but you can still add it to your `Cargo.toml` using,

```toml
stencila = { git = "https://github.com/stencila/stencila" }
```

## 🚀 Use

Open up the docs to see what's available (including re-exports),

```bash
cargo docs --open --package stencila
```

Then, use what you need :) e.g.

```rust
use stencila::{config, serve, tracing};
```

## 🛠️ Develop

### Getting started

Get started by cloning this repository and installing Cargo plugins (for linting, code coverage etc):

```sh
git clone git@github.com:stencila/stencila
cd stencila/rust
make setup
```

If you are contributing code please run formatting, linting and tests before submitting PRs:

```sh
make format lint test
```

To reduce compile times, we recommend using [`sccache`](https://github.com/mozilla/sccache), e.g

```sh
cargo install sccache
export RUSTC_WRAPPER=sccache
```

> 📢 It is likely that we will split off some of the modules (e.g. `methods/encode`) into their own sub-crate to improve compile times further.

### Testing

We make extensive use of Rust feature flags. The main benefit of this is reduced compile times during development. To take advantage of this use the Cargo options `--no-default-features` (to turn off all the default features) and `--features` (to turn on the features that you are currently working on).

For example, if you are working on the compilation of R code chunks, instead of compiling and running the relevant using tests like this:

```sh
cargo test compile::code::r
```

You should get faster compile times using this:

```sh
cargo test --no-default-features --features=compile-code-r compile::code::r
```

This approach is particularly useful when using `cargo watch`, for example to run the encode-decode ("ende") roundtrip integration test for HTML only when any source file changes:

```sh
cargo watch -x "test --no-default-features --features encode-html,decode-html --test ende html"
```

### Benchmarking

Run benchmarks using,

```sh
cargo bench
```

### Language queries

When developing `tree-sitter` language queries for the `methods::compile::code` module, the `tree-sitter` CLI is very useful.

1. Install and setup `tree-sitter` (this is a [good guide](https://dcreager.net/tree-sitter/getting-started/) to that.)

2. Parse fixture files to glean the structure of the AST for the language e.g.

    ```sh
    tree-sitter parse ../fixtures/fragments/r/imports.R
    ```

3. Create a query (or part of a larger query) and test it against the query files e.g.

    ```sh
    tree-sitter query query.txt ../fixtures/fragments/r/imports.R
    ```
