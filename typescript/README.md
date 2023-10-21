# Stencila Types

**JavaScript classes and TypeScript types for the Stencila Schema**

<a href="https://www.npmjs.com/package/@stencila/types">
  <img src="https://img.shields.io/npm/v/%40stencila%2Ftypes.svg?logo=npm&label=%40stencila%2Ftypes&&style=for-the-badge&color=1d3bd1&logoColor=66ff66&labelColor=3219a8">
</a>

## 👋 Introduction

This package provides JavaScript classes and TypeScript types for the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme).

Its main purpose is to allow functions in the [`@stencila/node`](https://github.com/stencila/stencila/tree/main/node) package to consume and return documents that are strongly typed. For example, with this package you could,

- construct documents programmatically using TypeScript and use `@stencila/node` to write them to multiple formats (e.g. Markdown, JATS XML, PDF)

- read existing documents from disk using `@stencila/node` and use TypeScript to render them in the browser

## 📦 Install

```console
npm install @stencila/types
```

```console
yarn add @stencila/types
```

```console
pnpm add @stencila/types
```

## ⚡ Usage

## Object types

Object types (aka product types) in the Stencila Schema are represented as JavaScript classes. The constructor for these classes has required properties as the initial parameters, and a final `options` parameter for all other properties.

For example, to construct a document with a single "Hello world!" paragraph, you can construct `Article`, `Paragraph` and `Text` with required properties only:

```js
import { CreativeWork, Article, Paragraph, Text, Thing } from "@stencila/types";

const doc = new Article([new Paragraph([new Text("Hello world!")])]);

doc instanceof Article; // true
doc instanceof CreativeWork; // true
doc instanceof Thing; // true

doc.content[0] instanceof Paragraph; // true

doc.content[0].content[0] instanceof Text; // true
```

Pass optional properties, in the final argument to the constructor. For example, to add an author to the article:

```js
import {
  Article,
  Organization,
  Paragraph,
  Person,
  Text,
} from "@stencila/types";

const doc = new Article([new Paragraph([new Text("Hello world!")])], {
  authors: [
    new Person({
      givenNames: ["Alice"],
      familyNames: ["Alvarez"],
      affiliations: [
        new Organization({
          name: "Aardvark University",
        }),
      ],
    }),
  ],
});
```

Alternatively, you may prefer to use the factory functions that are defined for each class (using the camelCased name of the type). This avoids having to type `new` and is a little more readable:

```js
import {
  article,
  organization,
  paragraph,
  person,
  text,
} from "@stencila/types";

const doc = article([paragraph([text("Hello world!")])], {
  authors: [
    person({
      givenNames: ["Alice"],
      familyNames: ["Alvarez"],
      affiliations: [
        organization({
          name: "Aardvark University",
        }),
      ],
    }),
  ],
});
```

### Union types

Union types (aka sum types) in the Stencila Schema are represented as TypeScript discriminated unions. For example, the `Block` union type is defined like so:

```ts
export type Block =
  Call |
  Claim |
  CodeBlock |
  CodeChunk |
  Division |
  Figure |
  For |
  Form |
  Heading |
  ...
```

In addition, for each union type a factory function is defined (again, using the camelCased name of the type). This function will, if necessary, hydrate plain JavaScript objects into the corresponding class (based on the `type` property). e.g.

```ts
import { block, paragraph, Paragraph, subscript } from "@stencila/types";

const p1 = block({
  type: "Paragraph",
  content: [],
});
p1 instanceof Paragraph; // true

const p2 = block(paragraph([]));
p2 instanceof Paragraph; // true

block(subscript([])); // errors because `Subscript` is not a `Block`
```

### Enumeration types

Enumeration types in the Stencila Schema are represented as TypeScript literal unions. For example, the `CitationIntent` enumeration is defined like so:

```ts
export type CitationIntent =
  'AgreesWith' |
  'CitesAsAuthority' |
  'CitesAsDataSource' |
  'CitesAsEvidence' |
  'CitesAsMetadataDocument' |
  'CitesAsPotentialSolution' |
  'CitesAsRecommendedReading' |
  ...
```

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

There is a `npm run check` for checking aspects of packaging. At present, CommonJS modules are not supported, only ESM.

So that debuggers and other tools can show the original source code, `declarationMap` and `sourceMap` are turned on in `tsconfig.json` and `src` is included in the `files` option of `package.json`.
