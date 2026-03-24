---
title: JSON
description: JavaScript Object Notation
---

# Introduction

[JavaScript Object Notation (JSON)](https://www.json.org/) is a lightweight data interchange format defined by [ECMA-404](https://ecma-international.org/publications-and-standards/standards/ecma-404/) and widely used for structured data storage and transmission. JSON's simplicity and broad language support make it a common choice for APIs, configuration files, and data exchange.

Stencila uses JSON as the default storage format for documents. Conversion is lossless — all Stencila Schema node types and properties are preserved through round-trips.

# Usage

Use the `.json` file extension, or the `--to json` or `--from json` options, when converting to/from JSON e.g.

```sh
stencila convert doc.smd doc.json
```

By default, the encoded JSON is indented. The `--compact` option can be used to produce un-indented, single line JSON.

When the `--standalone` option is used (the default for encoding to files), two properties are added to the JSON encoding of root nodes to improve interoperability:

- a `$schema` property which links to the [JSON Schema](https://json-schema.org) for the node type
- a `@context` property which links to the [JSON-LD](https://json-ld.org) context for the Stencila Schema

For example,

```json
{
  "$schema": "https://stencila.org/Article.schema.json",
  "@context": "https://stencila.org/context.jsonld",
  "type": "Article",
  ...
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and JSON powered by [`serde_json`](https://crates.io/crates/serde_json).

# Limitations

- JSON files for complex documents can be large. For smaller files, consider [CBOR](../cbor), [CBOR+Zstd](../cborzstd), or [JSON+Zip](../jsonzip).
- JSON lacks support for comments. For a lossless format that supports comments, use [JSON5](../json5).
