---
title: LaTeX
description: A typesetting system for technical documents
---
# Introduction

[LaTeX](https://www.latex-project.org/) is a document preparation system widely used for scientific and technical writing.

# Usage

Use the `.latex` file extension, or the `--to latex` or `--from latex` options, when converting to/from LaTeX e.g.

```sh
stencila convert doc.smd doc.latex
```

# Implementation

LaTeX support is implemented in the Rust crate [`codec-latex`](https://github.com/stencila/stencila/blob/main/rust/codec-latex).

# Notes

- LaTeX conversion is lossy for elements without LaTeX equivalents.
- For full TeX input, use the `tex` format.
