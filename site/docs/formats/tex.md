---
title: TeX
description: Plain TeX format
---

# Introduction

[TeX](https://tug.org/) is the underlying typesetting system created by Donald Knuth that LaTeX and other macro packages build upon. Stencila supports the `.tex` file extension for documents that use TeX conventions.

In practice, most TeX files use LaTeX macros. The distinction in Stencila is primarily about file extension: `.tex` files are handled identically to [LaTeX](../latex) files by the same codec.

# Usage

Use the `.tex` file extension, or the `--to tex` or `--from tex` options, when converting to/from TeX e.g.

```sh
stencila convert doc.smd doc.tex
```

# Implementation

TeX support is provided by the same Rust crate as LaTeX: [`codec-latex`](https://github.com/stencila/stencila/blob/main/rust/codec-latex). See the [LaTeX](../latex) documentation for full implementation details.

# Limitations

- The same limitations as [LaTeX](../latex) apply.
- Plain TeX primitives without LaTeX abstractions are not specifically handled; the codec assumes LaTeX-style markup.
