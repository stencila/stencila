---
title: JSON5
description: A more human readable flavor of JSON
---
# Introduction

[JSON5](https://json5.org/) is an extension of the JSON (JavaScript Object Notation) format that incorporates additional features for enhanced readability and flexibility. It maintains compatibility with standard JSON while introducing human-friendly syntax elements such as comments, trailing commas, and relaxed quoting rules.

Stencila provides support for JSON5 as a more human-readable, while still lossless, alternative to [JSON](../json) for storing documents. JSON5 is also used internally within Stencila as a more human-friendly, JavaScript-like way to represent nodes within documents in formats such as Markdown.

# Usage

Use the `.json5` file extension, or the `--to json5` or `--from json5` options, when converting to/from JSON5 e.g.

```sh
stencila convert doc.smd doc.json5
```

By default, the encoded JSON5 is indented. The `--compact` option can be used to produce un-indented, single line JSON5.

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and JSON5. The [`codec-json5-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-json5-trait) Rust crate implements `from_json5` and `to_json5` methods (and variants of those) for all node types in Stencila Schema, powered by [`json5`](https://crates.io/crates/json5) and [`json5format`](https://crates.io/crates/json5format).

# Notes

- The `--compact` option produces single-line JSON5 output.
- JSON5 preserves all Stencila schema data in a human-friendly syntax.
