---
title: CSL-JSON
description: Citation Style Language in JSON form
---
# Introduction

[CSL-JSON](https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html) is a JSON serialization of Citation Style Language data, commonly used for bibliographic entries and citations.

# Usage

Use the `.csl` file extension, or the `--from csl` option, when converting from CSL-JSON e.g.

```sh
stencila convert references.csl doc.smd
```

# Implementation

CSL-JSON decoding is implemented in the Rust crate [`codec-csl`](https://github.com/stencila/stencila/blob/main/rust/codec-csl).

# Notes

- CSL-JSON is currently supported for import only.
- CSL-JSON maps to Stencila bibliographic node types and may drop fields that are not in the schema.
