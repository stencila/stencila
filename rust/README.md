# Stencila Rust

**Core Stencila functionality implemented in a fast, memory-safe language**

## 🦀 Introduction

This is the `stencila` core Rust library. Its main purpose is to implement core functionality that can be reused in the Stencila CLI and in language bindings.

## 📦 Install

You'll need to have [Rust installed](https://rustup.rs) first.

This library is not yet published as a Rust crate, but you can still add it to your `Cargo.toml` using,

```toml
stencila = { git = "https://github.com/stencila/stencila" }
```

## 🛠️ Develop

### Getting started

Get started by cloning this repository and [installing Rust](https://rustup.rs) and necessary Cargo plugins (for formatting, linting, etc):

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

This library is made up of several Rust crates. Most of these crates are internal and are not published (indicated with `version = "0.0.0"`). Splitting code into many small creates has advantages for compilation speed, modularization and reuse.
