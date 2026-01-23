---
title: CBOR
description: Concise Binary Object Representation
---
# Introduction

The [CBOR (Concise Binary Object Representation)](https://cbor.io/) format is a binary serialization format suitable when speed and storage efficiency are important. It follows the data model of JSON closely, and like JSON, it enables data interchange without needing a formally specified schema. For further details about CBOR, see [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) and other [CBOR specs](https://cbor.io/spec.html).

Stencila provides support for CBOR as a faster, more compact, alternative to [JSON](../json) for storing documents.

# Usage

Use the `.cbor` file extension, or the `--to cbor` or `--from cbor` options, when converting to/from CBOR e.g.

```sh
stencila convert doc.smd soc.cbor
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and CBOR powered by the Rust crate [`ciborium`](https://crates.io/crates/ciborium).

# Notes

- CBOR is a binary alternative to JSON, optimized for size and speed.
- Round-tripping is lossless for supported nodes.
