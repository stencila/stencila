---
title: CSL-JSON
description: Citation Style Language in JSON form
---

# Introduction

[CSL-JSON](https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html) is a JSON serialization of Citation Style Language data, widely used for exchanging bibliographic entries and citation metadata between reference managers and document tools.

# Usage

Use the `.csl` file extension, or the `--from csl` option, when importing from CSL-JSON e.g.

```sh
stencila convert references.csl doc.smd
```

> [!info]
> CSL-JSON is currently supported for import (decoding) only.

# Implementation

CSL-JSON decoding is implemented in the Rust crate [`codec-csl`](https://github.com/stencila/stencila/blob/main/rust/codec-csl). Bibliographic entries are mapped to Stencila node types such as `Article`, `Book`, and `Person`.

# Limitations

- Export (encoding) to CSL-JSON is not supported.
- CSL-JSON fields that do not map to Stencila Schema properties are dropped during import.
- Only a subset of CSL item types are mapped; uncommon types may be imported as generic `CreativeWork` nodes.
