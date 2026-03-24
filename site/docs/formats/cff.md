---
title: Citation File Format
description: Citation metadata for software and research artifacts
---

# Introduction

The [Citation File Format (CFF)](https://citation-file-format.github.io/) is a YAML-based format for providing citation metadata alongside software repositories and research outputs. It is commonly used via `CITATION.cff` files in GitHub repositories.

# Usage

Use the `.cff` file extension, or the `--from cff` option, when importing from CFF e.g.

```sh
stencila convert CITATION.cff doc.smd
```

> [!info]
> CFF is currently supported for import (decoding) only.

# Implementation

CFF decoding is implemented in the Rust crate [`codec-cff`](https://github.com/stencila/stencila/blob/main/rust/codec-cff) using [`serde_yaml`](https://crates.io/crates/serde_yaml). CFF entries are mapped primarily to `CreativeWork` and `Person` nodes in Stencila Schema.

# Limitations

- Export (encoding) to CFF is not supported.
- CFF fields that do not have Stencila Schema equivalents (e.g. `repository-artifact`, `license-url`) are dropped during import.
- Only CFF version 1.2.0 structure is supported.
