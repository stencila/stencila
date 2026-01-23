---
title: YAML
description: A human-readable data serialization format
---
# Introduction

[YAML (YAML Ain't Markup Language)](https://yaml.org/) is a human-readable data serialization format commonly used for configuration files and data representation. It is known for its simplicity and readability, making it a preferred choice for settings and data structures. YAML's structure is based on indentation, allowing users to represent data hierarchies in an easily understandable manner.

Stencila provides support for YAML as a more human-readable, while still lossless, alternative to [JSON](../json) for storing documents.

# Usage

Use the `.yaml` file extension, or the `--to yaml` or `--from yaml` options, when converting to/from YAML e.g.

```sh
stencila convert doc.smd doc.yaml
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and YAML. The [`codec-yaml`](https://github.com/stencila/stencila/blob/main/rust/codec-yaml) Rust crate implements `from_yaml` and `to_yaml` methods for all node types in Stencila Schema, powered by [`serde_yaml`](https://crates.io/crates/serde_yaml).

When the `--standalone` option is used (the default for encoding to files), two properties are added to the YAML encoding of root nodes to improve interoperability:

- a `$schema` property which links to the [JSON Schema](https://json-schema.org) for the node type
- a `@context` property which links to the [JSON-LD](https://json-ld.org) context for the Stencila Schema

For example,

```yaml
$schema: https://stencila.org/Article.schema.json
"@context": https://stencila.org/context.jsonld
type: Article
```

# Notes

- YAML preserves all Stencila schema data in a human-friendly syntax.
- Round-tripping is lossless for supported nodes.
