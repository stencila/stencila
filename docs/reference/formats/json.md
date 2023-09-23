# JSON

## Introduction

[JSON (JavaScript Object Notation)](https://www.json.org/) is a lightweight data interchange format widely used for structured data storage and transmission. It is easy for both humans and machines to read and write. JSON's simplicity, flexibility, and compatibility with various programming languages make it a popular choice for APIs, configuration files, and data exchange between applications. 

Its benefits include simplicity, and support for nested data structures, making it a good choice for lossless serialization of Stencila documents for inter-application communication.

## Implementation

Stencila support lossless, bi-directional conversion between Stencila documents and JSON. The `codec-json` Rust crate implements `from_json` and `to_json` methods (and variants of those) for all node types in Stencila Schema, powered by [`serde_json`](https://crates.io/crates/serde_json). 

## Encodings

By default, the encoded JSON is indented but the `--compact` option is supported which produces un-indented, single line JSON.

When the `--standalone` option is used (the default for encoding to files), two properties are added to the JSON encoding of root nodes to improve interoperability:

- a `$schema` property which links to the [JSON Schema](https://json-schema.org) for the node type
- a `@context` property which links to the [JSON-LD](https://json-ld.org) context for the node type

For example,

```json
{
  "$schema": "https://stencila.dev/Article.schema.json",
  "@context": "https://stencila.dev/Article.jsonld",
  "type": "Article",
  ...
```

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->
<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
