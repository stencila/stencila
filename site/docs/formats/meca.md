---
title: Meca
description: Manuscript Exchange Common Approach package
---
# Introduction

[MECA](https://www.niso.org/standards-committees/meca) is a package format for exchanging manuscripts and related files in scholarly publishing workflows.

# Usage

Use the `.meca` file extension, or the `--from meca` option, when converting from a MECA package e.g.

```sh
stencila convert article.meca doc.smd
```

# Implementation

MECA decoding is implemented in the Rust crate [`codec-meca`](https://github.com/stencila/stencila/blob/main/rust/codec-meca).

# Notes

- MECA is currently supported for import only.
- MECA packages may include multiple files; Stencila extracts the primary article content.
