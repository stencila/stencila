---
title: JSON+Zip
description: JavaScript Object Notation with Zip archive compression
---

# Introduction

This format combines [JSON](../json) with the [Zip](https://en.wikipedia.org/wiki/ZIP_(file_format)) compressed archive format.

Stencila supports JSON+Zip as a portable compressed alternative to [JSON](../json) for storing large documents. The JSON content inside the archive can be inspected with standard Zip tools, making it more portable than [CBOR+Zstd](../cborzstd), though CBOR+Zstd may achieve better compression ratios.

# Usage

Use the `.json.zip` file extension, or the `--to jsonzip` or `--from jsonzip` options, when converting to/from JSON+Zip e.g.

```sh
stencila convert doc.smd doc.json.zip
```

# Implementation

Stencila supports lossless, bi-directional conversion between Stencila documents and JSON+Zip powered by the Rust crates [`serde_json`](https://crates.io/crates/serde_json) and [`zip`](https://crates.io/crates/zip).

# Limitations

- Compression adds overhead for small documents where the Zip metadata may negate size savings.
- The archive contains a single JSON file. It does not bundle external assets such as images.
