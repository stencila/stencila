---
title: PNG
description: Portable Network Graphics
---

# Introduction

PNG (Portable Network Graphics) is a lossless image format widely used for screenshots, graphics, and web images. Stencila can render documents or individual nodes to PNG for sharing, embedding, or use as static previews.

# Usage

Use the `.png` file extension, or the `--to png` option, when converting to PNG e.g.

```sh
stencila convert doc.smd doc.png
```

# Implementation

PNG encoding is implemented in the Rust crate [`codec-png`](https://github.com/stencila/stencila/blob/main/rust/codec-png). By default, nodes are encoded to LaTeX and then rendered to a PNG image. Image resizing and optimization options are also supported for use cases like email embedding.

# Limitations

- PNG is encode-only. Stencila cannot reconstruct document structure from PNG images.
- Rendering relies on a LaTeX installation being available.
- The output is a flat image with no selectable text or document structure.
- Very long documents may produce extremely tall images.
