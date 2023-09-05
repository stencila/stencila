# JSON5

## Introduction

[JSON5](https://json5.org/) is an extension of the JSON (JavaScript Object Notation) format that incorporates additional features for enhanced readability and flexibility. It maintains compatibility with standard JSON while introducing human-friendly syntax elements such as comments, trailing commas, and relaxed quoting rules. JSON5 is a good choice for configuration files and data serialization, especially when human readability is a priority. 

Its benefits include improved readability, support for comments and relaxed syntax rules. It is a good choice for lossless serialization of Stencila documents when human-readability is important.

## Implementation

Stencila support lossless, bi-directional conversion between Stencila documents and JSON5.  The `codec-json5-trait` Rust crate implements `from_json5` and `to_json5` methods (and variants of those) for all node types in Stencila Schema, powered by the [`json5`](https://crates.io/crates/json5) and [`json5format`](https://crates.io/crates/json5format).

## Encodings

By default, the encoded JSON5 is indented but the `--compact` option is supported which produces un-indented JSON5.
