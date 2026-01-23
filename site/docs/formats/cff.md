---
title: Citation File Format
description: Citation metadata for software and research artifacts
---
# Introduction

The [Citation File Format (CFF)](https://citation-file-format.github.io/) is a YAML-based format for sharing citation metadata with software repositories and research outputs.

# Usage

Use the `.cff` file extension, or the `--from cff` option, when converting from CFF e.g.

```sh
stencila convert CITATION.cff doc.smd
```

# Implementation

CFF decoding is implemented in the Rust crate [`codec-cff`](https://github.com/stencila/stencila/blob/main/rust/codec-cff) using [`serde_yaml`](https://crates.io/crates/serde_yaml).

# Notes

- CFF is currently supported for import only.
- CFF primarily maps to `CreativeWork` and `Person` metadata in Stencila.
