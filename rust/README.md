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

Get started by cloning this repository and building the library:

```sh
git clone git@github.com:stencila/stencila
cd stencila/rust
make build
```

If you are contributing code please run formatting, linting and tests before submitting PRs:

```sh
make format lint test
```
