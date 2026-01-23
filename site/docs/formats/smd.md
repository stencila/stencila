---
title: Stencila Markdown
description: An extended flavor of Markdown supporting Stencila's document schema
---
# Introduction

Stencila Markdown provides extensions to Markdown to support document elements not natively supported by Markdown.

# Usage

To convert to/from Stencila Markdown, use the `.smd` file extension, or the `--to smd` or `--from smd` options e.g.

```sh
stencila convert doc.docx doc.smd
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Stencila Markdown. See our [CommonMark](../md) documentation for implementation details.

# Notes

- Stencila Markdown adds block and inline syntax to represent Stencila nodes in text.
- Some non-Markdown features are encoded with Stencila-specific extensions.
