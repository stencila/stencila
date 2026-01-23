---
title: PNG
description: Portable Network Graphics
---
# Introduction

PNG is a lossless image format commonly used for screenshots and graphics. Stencila can render documents or nodes to PNG for sharing and embedding.

# Usage

Use the `.png` file extension, or the `--to png` option, when converting to PNG e.g.

```sh
stencila convert doc.smd doc.png
```

# Implementation

PNG output is implemented in the Rust crate [`codec-png`](https://github.com/stencila/stencila/blob/main/rust/codec-png).

# Notes

- PNG output is one-way and does not preserve document structure.
- Rendering relies on HTML and LaTeX conversions behind the scenes.
