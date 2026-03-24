---
title: Email HTML
description: HTML optimized for email clients
---

# Introduction

Email HTML is a specialized HTML output format optimized for rendering in email clients. Email clients have limited and inconsistent HTML/CSS support, so this format uses table-based layouts and inline styles to maximize compatibility across clients like Gmail, Outlook, and Apple Mail.

Stencila generates Email HTML by first encoding documents to [MJML](../mjml), then rendering the MJML to HTML.

# Usage

Use the `.email.html` file extension, or the `--to email` option, when converting to Email HTML e.g.

```sh
stencila convert doc.smd doc.email.html
```

# Implementation

Email HTML encoding is implemented in the Rust crate [`codec-email`](https://github.com/stencila/stencila/blob/main/rust/codec-email). Documents are first encoded to MJML markup, then rendered to HTML using the [`mrml`](https://crates.io/crates/mrml) crate. Only `Article` nodes are supported as root nodes.

# Limitations

- Email HTML is encode-only. It cannot be parsed back into Stencila documents.
- Interactive elements (e.g. code execution widgets, interactive figures) are not supported in email output.
- Complex layouts, wide tables, and large images may not render well across all email clients.
- Only `Article` nodes can be encoded; other root node types will produce an error.
