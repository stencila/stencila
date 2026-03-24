---
title: DOM HTML
description: A verbose HTML representation of Stencila document nodes
---

# Introduction

DOM HTML is an internal HTML representation used by Stencila for rendering and previewing documents. It uses custom HTML elements (e.g. `<stencila-paragraph>`, `<stencila-code-chunk>`) to preserve the full structure of Stencila document nodes, including properties that plain HTML cannot represent.

DOM HTML is the format used by Stencila's web-based document viewer and editor.

# Usage

Use the `.dom.html` file extension, or the `--to dom` option, when converting to DOM HTML e.g.

```sh
stencila convert doc.smd doc.dom.html
```

# Implementation

DOM HTML encoding is implemented in the Rust crate [`codec-dom`](https://github.com/stencila/stencila/blob/main/rust/codec-dom), with per-node-type encoding derived via [`codec-dom-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-dom-trait) and [`codec-dom-derive`](https://github.com/stencila/stencila/blob/main/rust/codec-dom-derive).

# Limitations

- DOM HTML is encode-only. It cannot be parsed back into Stencila documents.
- The output is intentionally verbose and not intended for hand-editing or general web publishing. Use [HTML](../html) for standard web output.
- Custom elements require Stencila's JavaScript runtime for interactive behavior.
