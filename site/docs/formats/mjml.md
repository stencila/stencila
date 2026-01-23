---
title: MJML
description: Markup for responsive email templates
---
# Introduction

[MJML](https://mjml.io/) is a markup language for responsive email templates. Stencila can emit MJML as an intermediate format for email rendering.

# Usage

Use the `.mjml` file extension, or the `--to mjml` option, when converting to MJML e.g.

```sh
stencila convert doc.smd doc.mjml
```

# Implementation

MJML output is implemented in the Rust crate [`codec-email`](https://github.com/stencila/stencila/blob/main/rust/codec-email).

# Notes

- MJML output is intended for email pipelines and is output-only.
- Render MJML to HTML using Stencila's Email HTML format or an MJML renderer.
