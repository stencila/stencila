---
title: CBOR
description: Concise Binary Object Representation
---

# Introduction

The [CBOR (Concise Binary Object Representation)](https://cbor.io/) format is a binary serialization format defined in [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html). It follows the data model of JSON closely but uses a compact binary encoding, making it faster to parse and smaller to store than text-based formats. For further details, see the [CBOR specification](https://cbor.io/spec.html).

Stencila supports CBOR as a faster, more compact alternative to [JSON](../json) for storing and transferring documents. Like JSON, conversion is lossless — all Stencila Schema node types and properties are preserved through round-trips.

# Usage

Use the `.cbor` file extension, or the `--to cbor` or `--from cbor` options, when converting to/from CBOR e.g.

```sh
stencila convert doc.smd doc.cbor
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and CBOR powered by the Rust crate [`ciborium`](https://crates.io/crates/ciborium).

# Limitations

- CBOR files are not human-readable. For a human-readable lossless format, use [JSON](../json), [JSON5](../json5), or [YAML](../yaml).
- No built-in compression. For smaller files, use [CBOR+Zstd](../cborzstd) which adds Zstandard compression.
