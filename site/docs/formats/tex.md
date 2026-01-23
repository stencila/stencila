---
title: TeX
description: Plain TeX format
---
# Introduction

TeX is the underlying typesetting system used by LaTeX and related formats.

# Usage

Use the `.tex` file extension, or the `--to tex` or `--from tex` options, when converting to/from TeX e.g.

```sh
stencila convert doc.smd doc.tex
```

# Implementation

TeX support is implemented in the Rust crate [`codec-latex`](https://github.com/stencila/stencila/blob/main/rust/codec-latex).

# Notes

- TeX conversion is lossy for Stencila nodes without TeX equivalents.
- TeX output focuses on typesetting rather than document semantics.
