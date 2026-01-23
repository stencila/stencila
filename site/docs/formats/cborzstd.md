---
title: CBOR+ZStd
description: Concise Binary Object Representation with ZStandard Compression
---
# Introduction

This format combines [Concise Binary Object Representation (CBOR)](../cbor) and [ZStandard](http://facebook.github.io/zstd/), a fast lossless compression algorithm.

Stencila provides support for CBOR+ZStd as a more compact alternative to [JSON](../json) or [CBOR](../cbor) for storing documents. It may be preferred over those formats for storing very large documents. Also consider [JSON+Zip](../jsonzip) which may provide similar levels of compression, but which is more portable.

# Usage

Use the `.cbor.zstd` file extension, or the `--to cborzstd` or `--from cborzstd` options, when converting to/from CBOR+Zstd e.g.

```sh
stencila convert doc.smd doc.cbor.zstd
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and CBOR+ZStd powered by the Rust crates [`ciborium`](https://crates.io/crates/ciborium) and [`zstd`](https://crates.io/crates/zstd).

# Notes

- CBOR+ZStd is a compressed variant of CBOR for smaller files.
- Round-tripping is lossless for supported nodes.
