---
title: R+LaTeX
description: Sweave/knitr Rnw format
---
# Introduction

Rnw files combine LaTeX with embedded R code chunks for literate programming workflows.

# Usage

Use the `.rnw` file extension, or the `--to rnw` or `--from rnw` options, when converting to/from Rnw e.g.

```sh
stencila convert doc.smd doc.rnw
```

# Implementation

Rnw support is implemented in the Rust crate [`codec-rnw`](https://github.com/stencila/stencila/blob/main/rust/codec-rnw).

# Notes

- Rnw conversion is lossy for nodes without LaTeX or R chunk equivalents.
- The format targets R-focused literate programming workflows.
