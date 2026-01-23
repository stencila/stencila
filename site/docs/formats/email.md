---
title: Email HTML
description: HTML optimized for email clients
---
# Introduction

Email HTML is a specialized HTML output optimized for email clients. Stencila generates this format via MJML to maximize compatibility.

# Usage

Use the `.email.html` file extension, or the `--to email` option, when converting to Email HTML e.g.

```sh
stencila convert doc.smd doc.email.html
```

# Implementation

Email HTML output is implemented in the Rust crate [`codec-email`](https://github.com/stencila/stencila/blob/main/rust/codec-email) using [`mrml`](https://crates.io/crates/mrml) to render MJML into HTML.

# Notes

- Email HTML is output-only and intentionally constrained for client compatibility.
- The output is lossy for interactive and layout-heavy elements.
