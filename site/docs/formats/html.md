---
title: HTML
description: HyperText Markup Language
---
# Introduction

HTML (HyperText Markup Language) is the standard markup language used to structure and display content on the web.

Stencila provides limited support for converting to and from HTML.

# Usage

Use the `.html` file extension, or the `--to html` or `--from html` options, when converting to/from HTML e.g.

```sh
stencila convert doc.smd doc.html
```

> [!warning]
> Stencila's HTML support is in alpha status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Parsing of HTML is largely done using the [quick-xml](https://crates.io/crates/quick-xml) Rust crate.

# Notes

- Stencila also supports a "DOM HTML" format for previewing documents that uses custom elements and is intentionally verbose.
- HTML conversion is lossy for unsupported node types and layout.
