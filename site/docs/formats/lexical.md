---
title: Lexical JSON
description: Serialization format for the Lexical editor framework
---

# Introduction

[Lexical](https://lexical.dev) is a modern, extensible text editor framework developed by Meta for building rich text editing experiences in web applications. [Lexical JSON](https://lexical.dev/docs/concepts/serialization) is the serialization format for documents created with Lexical-based editors.

Stencila supports conversion to/from Lexical JSON to enable integration with Lexical-based editors and content management systems.

# Usage

Use the `.lexical` file extension, or the `--to lexical` or `--from lexical` options, when converting to/from Lexical JSON e.g.

```sh
stencila convert doc.smd doc.lexical
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Lexical JSON, implemented in the Rust crate [`codec-lexical`](https://github.com/stencila/stencila/blob/main/rust/codec-lexical). The codec uses [`serde_json`](https://crates.io/crates/serde_json) with custom transformer functions to map between Lexical node types and Stencila Schema node types.

# Limitations

- Lexical focuses on rich text editing. Stencila node types that go beyond rich text (e.g. executable code chunks, math blocks, data tables) have no Lexical equivalent and are lost during export.
- Custom Lexical node types from third-party plugins are not recognized during import.
- Lexical's editor state metadata (selection, history) is not preserved.
