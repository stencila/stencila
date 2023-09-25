# Stencila Types

**JavaScript classes and TypeScript types for the Stencila Schema**

## 👋 Introduction

This package provides JavaScript classes and TypeScript types for the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme).

## 📦 Install

```console
npm i @stencila/types
```

## ⚡ Usage

The main purpose for this package is to provide TypeScript types corresponding to types in the Stencila Schema. This allows functions in the `@stencila/node` package to consume and return documents that are strongly typed.

You can construct a new document, conforming to the Stencila Schema, using the classes provided. For example, to construct an `Article` with a single "Hello world!" paragraph:

```js
import { Article, Paragraph, Text } from "@stencila/types";

const doc = new Article([new Paragraph([new Text("Hello world!")])]);

doc instanceof Article; // true
doc.content[0] instanceof Paragraph; // true
doc.content[0].content[0] instanceof Text; // true
```

Alternatively, you can pass JavaScript objects (perhaps parsed from JSON) to the `from` method of each class. However, note that in this case the child nodes will not be class instances:

```js
import { Article } from "@stencila/types";

const doc = Article.from({
  content: [{ content: [{ value: "Hello world!" }] }],
});

doc instanceof Article; // true
doc.content[0] instanceof Paragraph; // false
doc.content[0].content[0] instanceof Text; // false
```

There are also `*From` functions provided for the union types in the schema e.g. `nodeFrom`, `blockFrom`, `inlineFrom`. These functions will delegate to the corresponding class' constructor based on the `type` property:

```js
import { nodeFrom } from "@stencila/types";

const doc = nodeFrom({
  type: "Article",
  content: [{ content: [{ value: "Hello world!" }] }],
});

doc instanceof Article; // true
```

At this stage, this package does not do any validation of objects passed to the 'from' functions. This may be added in the future e.g. using `ajv`.

## 🛠️ Develop

Most of the types are generated from the Stencila Schema by the Rust [`schema-gen`](https://github.com/stencila/stencila/tree/main/rust/schema-gen#readme) crate. See there for contributing instructions.

### Linting and testing

Please run linting checks and tests before contributing any code changes.

```sh
npm run lint
npm test

# or

make lint test
```

### Packaging

Some notes on packaging:

- There is a `npm run check` for checking aspects of packaging

- At present, CommonJS modules are not supported, only ESM.

- So that debuggers and other tools can show the original source code, `declarationMap` and `sourceMap` are turned on in `tsconfig.json` and `src` is including in `package.json`.
