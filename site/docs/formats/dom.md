---
title: DOM HTML
description: A verbose HTML representation of Stencila nodes
---
# Introduction

DOM HTML is an internal HTML representation used by Stencila for rendering and previewing documents. It uses custom elements for Stencila node types and aims to preserve structure.

# Usage

Use the `.dom.html` file extension, or the `--to dom` option, when converting to DOM HTML e.g.

```sh
stencila convert doc.smd doc.dom.html
```

# Implementation

DOM HTML encoding is implemented in the Rust crate [`codec-dom`](https://github.com/stencila/stencila/blob/main/rust/codec-dom).

# Notes

- DOM HTML is intended for rendering and debugging, not hand editing.
- The format is verbose and includes custom elements such as `<stencila-paragraph>`.
