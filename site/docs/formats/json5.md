---
title: JSON5
description: A more human-readable flavor of JSON
---

# Introduction

[JSON5](https://json5.org/) is an extension of JSON that adds human-friendly syntax features including comments, trailing commas, unquoted keys, and multi-line strings. It maintains full compatibility with standard JSON while being easier to read and write by hand.

Stencila supports JSON5 as a more human-readable, lossless alternative to [JSON](../json) for storing documents. JSON5 is also used internally within Stencila as a concise, JavaScript-like way to represent node properties within Markdown-based formats.

# Usage

Use the `.json5` file extension, or the `--to json5` or `--from json5` options, when converting to/from JSON5 e.g.

```sh
stencila convert doc.smd doc.json5
```

By default, the encoded JSON5 is indented. The `--compact` option can be used to produce un-indented, single line JSON5.

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and JSON5. The [`codec-json5-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-json5-trait) Rust crate implements `from_json5` and `to_json5` methods for all node types in Stencila Schema, powered by [`json5`](https://crates.io/crates/json5) and [`json5format`](https://crates.io/crates/json5format).

# Limitations

- JSON5 is not as widely supported by tooling as standard JSON. Most JSON parsers cannot read JSON5.
- Comments added to JSON5 files are not preserved through Stencila round-trips.
