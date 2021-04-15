# Stencila Node.js

## 👋 Introduction

This is the `stencila` package for Node.js. It uses [`Neon`](https://neon-bindings.com/) to expose functionality implemented in the Stencila [Rust library](../rust) to Node.js. The main use of this package is to provide core functionality to the [Electron](https://www.electronjs.org/)-based [Stencila Desktop](../desktop) application.

## 📦 Install

```sh
npm install stencila
```

## 🚀 Use

```ts
import * as stencila from 'stencila'
```

See the in-code doc comment and the tests for more examples of usage.

See the Neon docs for advice on using in [Electron apps](https://neon-bindings.com/docs/electron-apps).

## 🛠️ Develop

Given this package is a set of bindings to the Stencila [Rust library](../rust) you'll need to have [Rust installed](https://rustup.rs) to build it. Then, get started by cloning this repository, installing dependencies and building the package:

```sh
git clone git@github.com:stencila/stencila
cd stencila/node
make setup build
```

Please run the formatting and linting tasks before contributing code e.g.

```sh
make format lint
```
