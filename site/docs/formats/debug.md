---
title: Debug
description: Rust debug representation of Stencila document nodes
---

# Introduction

The Debug format outputs Stencila documents using Rust's `Debug` trait formatting. This produces a verbose, structured representation that shows the exact types, enum variants, and field values of every node in the document tree.

This format is primarily useful for debugging and development — for example, to inspect which specific enum variants are present in a document or to verify that a codec is producing the expected internal representation.

# Usage

Use the `.debug` file extension, or the `--to debug` option, when converting to the Debug format e.g.

```sh
stencila convert doc.smd doc.debug
```

By default, the output is pretty-printed with indentation. Use the `--compact` option for single-line output:

```sh
stencila convert doc.smd doc.debug --compact
```

> [!info]
> Debug is encode-only. It is designed for inspecting document structure, not for importing documents.

# Implementation

Debug encoding is implemented in the Rust crate [`codec-debug`](https://github.com/stencila/stencila/blob/main/rust/codec-debug). It uses Rust's built-in `Debug` trait (`{:#?}` for pretty-printing, `{:?}` for compact output).

# Limitations

- Debug is encode-only. There is no corresponding decoder.
- The output format is Rust-specific and not intended to be parsed by other tools.
- The representation may change between Stencila versions as the schema evolves.
