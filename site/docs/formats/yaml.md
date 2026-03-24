---
title: YAML
description: A human-readable data serialization format
---

# Introduction

[YAML (YAML Ain't Markup Language)](https://yaml.org/) is a human-readable data serialization format commonly used for configuration files and data representation. Its indentation-based structure makes data hierarchies easy to read and write by hand.

Stencila supports YAML as a human-readable, lossless alternative to [JSON](../json) for storing documents.

# Usage

Use the `.yaml` file extension, or the `--to yaml` or `--from yaml` options, when converting to/from YAML e.g.

```sh
stencila convert doc.smd doc.yaml
```

When the `--standalone` option is used (the default for encoding to files), two properties are added to the YAML encoding of root nodes to improve interoperability:

- a `$schema` property which links to the [JSON Schema](https://json-schema.org) for the node type
- a `@context` property which links to the [JSON-LD](https://json-ld.org) context for the Stencila Schema

For example,

```yaml
$schema: https://stencila.org/Article.schema.json
"@context": https://stencila.org/context.jsonld
type: Article
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and YAML. The [`codec-yaml`](https://github.com/stencila/stencila/blob/main/rust/codec-yaml) Rust crate implements `from_yaml` and `to_yaml` methods for all node types in Stencila Schema, powered by [`serde_yaml`](https://crates.io/crates/serde_yaml).

# Limitations

- YAML's indentation-sensitive syntax can cause subtle errors if whitespace is inconsistent.
- Very large documents produce deeply nested YAML that can be difficult to navigate.
