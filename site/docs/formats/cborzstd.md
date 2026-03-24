---
title: CBOR+Zstd
description: Concise Binary Object Representation with Zstandard compression
---

# Introduction

This format combines [CBOR (Concise Binary Object Representation)](../cbor) with [Zstandard](http://facebook.github.io/zstd/), a fast lossless compression algorithm developed by Facebook.

Stencila supports CBOR+Zstd as the most compact storage option for documents. It is best suited for very large documents where file size matters. Also consider [JSON+Zip](../jsonzip) which offers similar compression ratios with broader tool compatibility.

# Usage

Use the `.cbor.zstd` file extension, or the `--to cborzstd` or `--from cborzstd` options, when converting to/from CBOR+Zstd e.g.

```sh
stencila convert doc.smd doc.cbor.zstd
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and CBOR+Zstd powered by the Rust crates [`ciborium`](https://crates.io/crates/ciborium) and [`zstd`](https://crates.io/crates/zstd).

# Limitations

- CBOR+Zstd files are not human-readable or inspectable without decompression.
- Requires Stencila or compatible tooling to decompress and deserialize. For a more portable compressed format, consider [JSON+Zip](../jsonzip).
