# Stencila SDK for Node.js

**Types and function bindings for using Stencila from Node.js**

<a href="https://www.npmjs.com/package/@stencila/node">
  <img src="https://img.shields.io/npm/v/%40stencila%2Fnode.svg?logo=npm&label=%40stencila%2Fnode&&style=for-the-badge&color=1d3bd1&logoColor=66ff66&labelColor=3219a8">
</a>

## 👋 Introduction

This package provides bindings to core [Stencila Rust](https://github.com/stencila/stencila/tree/main/rust#readme) functions. Most function take, or return, types defined in [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme) and transpiled to [@stencila/types](https://www.npmjs.com/package/@stencila/types).

The primary intended audience is developers who want to develop their own tools on top of Stencila's core functionality. For example, with this package you could construct Stencila documents programmatically using Node.js and write them to multiple formats (e.g. Markdown, JATS XML, PDF).

> [!IMPORTANT]
> At present, there are only bindings to functions for format conversion, but future versions will expand this scope to include document management (e.g branching and merging) and execution.

## 📦 Install

```console
npm install @stencila/node
```

## ⚡ Usage

### Conversion

The `convert` module has five functions for encoding and decoding Stencila documents and for converting documents between formats. All functions are `async`.

#### `fromString`

Use `fromString` to decode a string in a certain format to a Stencila Schema type. Usually you will need to supply the `format` argument (it defaults to JSON). e.g.

```ts
import { convert } from "@stencila/node";
import { Article, Paragraph, Text } from "@stencila/types";

const doc = await convert.fromString(
  '{type: "Article", content: [{type: "Paragraph", content: ["Hello world"]}]}',
  {
    format: "json5",
  },
);

doc instanceof Article; // true
doc instanceof CreativeWork; // true
doc instanceof Thing; // true

doc.content[0] instanceof Paragraph; // true

doc.content[0].content[0] instanceof Text; // true
```

#### `fromPath`

Use `fromPath` to decode a file system path (usually a file) to a Stencila Schema type. The format can be supplied, but if it is not, is inferred from the path. e.g.

```ts
import { convert } from "@stencila/node";

const doc = await convert.fromPath(
  "../examples/nodes/paragraph/paragraph.jats.xml",
);
```

#### `DecodeOptions`

Both `fromString` and `fromPath` accept a `DecodeOptions` object as the second argument with the options:

- `format: string`: The format to decode from

- `losses: string`: What to do if there are losses when decoding from the input. Possible values include `ignore`, `trace`, `debug`, `info`, `warn`, `error`, or `abort`, or a file path to write the losses to (`json` or `yaml` file extensions are supported).

#### `toString`

Use `toString` to encode a Stencila Schema type to a string. Usually you will want to supply the `format` argument (it defaults to JSON).

```ts
import { convert } from "@stencila/node";
import { Article, Paragraph, Text } from "@stencila/types";

const doc = new Article([
  new Paragraph(["Hello ", new Strong(["again"]), "!"]),
]);

const jats = await convert.toString(doc, { format: "jats" });
```

#### `toPath`

To encode a Stencila Schema type to a filesystem path, use `toPath`. e.g.

```ts
import { convert } from "@stencila/node";

const doc = new Article([new Paragraph([new Text("Hello file system!")])]);

await convert.toPath(doc, "doc.html", { compact: false });
```

#### `EncodeOptions`

Both `toString` and `toPath` accept a `EncodeOptions` object:

- `format: string`: The format to encode to

- `standalone: bool`: Whether to encode as a valid, standalone document (e.g. for HTML ensuring there is a root `<html>` element with a `<head>` and `<body>`). Unless specified otherwise, this is the default when encoding to a file.

- `compact: bool`: Whether to encode in compact form. Some formats (e.g HTML and JSON) can be encoded in either compact or "pretty-printed" (e.g. indented) forms.

- `losses: string`: What to do if there are losses when encoding to the output. Possible values include `ignore`, `trace`, `debug`, `info`, `warn`, `error`, or `abort`, or a file path to write the losses to (`json` or `yaml` file extensions are supported).

#### `fromTo`

Use `fromTo` when you want to convert a file to another format (i.e. as a more performant shortcut to combining `fromPath` and `toPath`).

```ts
import { convert } from "@stencila/node";

await convert.fromTo("doc.jats.xml", "doc.html");
```

The `fromTo` function accepts both `DecodeOptions` and `EncodeOptions` as third and fourth arguments respectively.

> [!NOTE]
> Some of the usage examples above illustrate manually constructing in-memory JavaScript representations of small documents. This is for illustration only and would be unwieldy for large documents. Instead we imagine developers using the `convert.fromString` or `convert.fromPath` functions to load documents into memory from other formats, or writing functions to construct documents composed of the Stencila classes.

## 🛠️ Develop

### Bindings

This packages uses [NAPI-RS](https://napi.rs) to generate a Node.js native addon from Stencila Rust functions. The addon (a binary file with a `.node` extension) is generated using `npm run build:addon` for releases and `npm run build:debug` for testing. The file `bindings.d.cts` is also generated by NAPI-RS.

NAPI-RS [recommends](https://napi.rs/docs/deep-dive/release) distributing native addons using different NPM packages for each platform supported. However, given how we are distributing other binaries (e.g. for the CLI), and to avoid the complexity of multiple NPM packages, we have opted for the "download in a postinstall phase" approach. See [`install.js`](install.js).

### `convert` module

The `convert` module is implemented in Rust (`src/convert.rs`) with a thin TypeScript wrapper (`src/convert.mts`) to provide documentation and conversion to the types in the `@stencila/types`.

When contributing code please run the following linting, formatting and testing scripts. Linting checks are run on CI, so for faster iteration, fewer failed runs and less noise, it's generally a good idea to run them locally before pushing code.

### Workspace dependencies

This module uses types from `@stencila/types`, a package that is defined in the sibling [`ts`](../ts) directory, and which is also part of the NPM [workspace](https://docs.npmjs.com/cli/v7/using-npm/workspaces) defined in the root [package.json](../package.json).

You may find that ESLint will complain that `@stencila/types` does not exist because [../ts/dist](../ts/dist) does not yet exist. To fix this build that package:

```console
cd ../ts && npm run build
```

or

```console
make -C ../ts build
```

### Linting && formatting

We use [ESLint](https://eslint.org/) and [Prettier](https://prettier.io/) for code linting and formatting respectively. To apply linting and formatting fixes:

```console
npm run fix
```

To just check linting and formatting:

```console
npm run lint
```

### Testing

We use [Jest](https://jestjs.io/) for tests. To run them:

```console
npm test
```

### Packaging

The packaging and publishing configuration is checked using [`arethetypeswrong``)(https://github.com/arethetypeswrong/arethetypeswrong.github.io) and [`publint`](https://publint.dev/):

```console
npm pubcheck
```

### `Makefile`

As with most modules in this repo, there is a `Makefile` which you may prefer to use for common development tasks. For example to easily run multiple NPM scripts at once:

```console
make fix test
```

A recommended combination of recipes to run before committing code is:

```console
make audit pubcheck lint test
```
