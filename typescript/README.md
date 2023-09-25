# Stencila Types

**JavaScript classes and TypeScript types for the Stencila Schema**

## 👋 Introduction

This is the `@stencila/types` NPM package which provides JavaScript classes and TypeScript types representing types in the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme).

## 📦 Install

```console
npm i @stencila/types
```

## ⚡ Usage

```ts
import { Article, Paragraph, Text } from "@stencila/types";

const doc = new Article([new Paragraph([new Text("Hello world!")])]);
```

## 🛠️ Develop

Most of the types are generated from the Stencila Schema by the Rust [`schema-gen`](https://github.com/stencila/stencila/tree/main/rust/schema-gen#readme) crate. See there for contributing instructions.

### Packaging

Some notes on packaging:

- There is a `npm run check` for checking aspects of packaging

- At present, CommonJS modules are not supported, only ESM.

- So that debuggers and other tools can show the original source code, `declarationMap` and `sourceMap` are turned on in `tsconfig.json` and `src` is including in `package.json`.
