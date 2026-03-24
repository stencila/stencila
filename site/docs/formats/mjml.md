---
title: MJML
description: Markup language for responsive email templates
---

# Introduction

[MJML](https://mjml.io/) is an open-source markup language for building responsive email templates. It abstracts away the complexity of email-compatible HTML by providing high-level components that compile to cross-client HTML.

Stencila can encode documents to MJML as an intermediate step in the [Email HTML](../email) pipeline, or as a standalone output for use with other MJML rendering tools.

# Usage

Use the `.mjml` file extension, or the `--to mjml` option, when converting to MJML e.g.

```sh
stencila convert doc.smd doc.mjml
```

To produce final email-ready HTML instead, use the [Email HTML](../email) format which renders MJML to HTML automatically.

# Implementation

MJML encoding is implemented in the Rust crate [`codec-email`](https://github.com/stencila/stencila/blob/main/rust/codec-email).

# Limitations

- MJML is encode-only. Stencila cannot import MJML files.
- Only `Article` nodes are supported as root nodes.
- Not all Stencila node types have MJML representations; unsupported nodes are omitted from the output.
