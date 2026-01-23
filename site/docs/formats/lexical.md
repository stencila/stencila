---
title: Lexical JSON
description: Format for integrating with Lexical-based editors
---
# Introduction

Facebook's [Lexical](https://lexical.dev) editor is a modern, extensible text editor framework designed for building interactive web applications with rich text capabilities. [Lexical JSON](https://lexical.dev/docs/concepts/serialization) is a serialization format for documents written using Lexical.

Stencila supports conversion to/from Lexical JSON as a way of integrating with Lexical and editors built on it.

# Usage

Use the `.lexical` file extension, or the `--to lexical` or `--from lexical` options, when converting to/from Lexical JSON e.g.

```sh
stencila convert doc.smd doc.lexical
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Lexical JSON. This is built on top of [`serde_json`](https://crates.io/crates/serde_json) with transformer functions to map between Lexical node types and Stencila node types.

# Notes

- Lexical JSON targets rich text editing; not all Stencila nodes have equivalents.
- Conversion is lossy for unsupported node types.
