---
title: JSON+Zip
description: JavaScript Object Notation with Zip Archive
---
# Introduction

This format combines [JavaScript Object Notation (JSON)](../json) and the [Zip](<https://en.wikipedia.org/wiki/ZIP_(file_format)>), compressed archive file format.

Stencila provides support for JSON+Zip as a more compact alternative to [JSON](../json) for storing large documents. Also consider [CBOR+Zstd](../cborzstd) which may provide better levels of compression.

# Usage

Use the `.json.zip` file extension, or the `--to jsonzip` or `--from jsonzip` options, when converting to/from JSON+Zip e.g.

```sh
stencila convert doc.smd doc.json.zip
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and JSON+Zip powered by the Rust crates [`serde_json`](https://crates.io/crates/serde_json) and [`zip`](https://crates.io/crates/zip).

# Notes

- JSON+Zip is useful when document size is a primary concern.
- Round-tripping is lossless for supported nodes.
