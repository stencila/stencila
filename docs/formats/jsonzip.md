---
title: JSON+Zip
description: JavaScript Object Notation with Zip Archive
config:
  publish:
    ghost:
      slug: jsonzip
      tags:
        - "#docs"
        - Formats
---

# Introduction

This format combines [JavaScript Object Notation (JSON)](../formats/json) and the [Zip](<https://en.wikipedia.org/wiki/ZIP_(file_format)>), compressed archive file format.

Stencila provides support for JSON+Zip as a more compact alternative to [JSON](../formats/json) for storing large documents. Also consider [CBOR+Zstd](../formats/cborzstd) which may provide better levels of compression.

# Usage

Use the `.json.zip` file extension, or the `--to jsonzip` or `--from jsonzip` options, when converting to/from JSON+Zip e.g.

```sh
stencila convert doc.smd doc.json.zip
```

# Implementation

Stencila support lossless, bi-directional conversion between Stencila documents and JSON+Zip powered by the Rust crates [`serde_json`](https://crates.io/crates/serde_json) and [`zip`](https://crates.io/crates/zip).

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
