---
title: Plain Text
description: A simple plain text representation of documents
---

# Introduction

Stencila can export documents to plain text, producing a flat text representation that strips all structural markup, styling, and metadata. This format is useful when you need raw text content for copy/paste, text analysis, or integration with tools that expect unformatted input.

# Usage

Use the `.txt` file extension, or the `--to text` option, when converting to plain text e.g.

```sh
stencila convert doc.smd doc.txt
```

> [!info]
> Plain text is encode-only. Stencila cannot reconstruct a structured document from a plain text file.

# Implementation

Plain text encoding is implemented in the Rust crate [`codec-text`](https://github.com/stencila/stencila/blob/main/rust/codec-text), using the text trait helpers in [`codec-text-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-text-trait).

# Limitations

- Plain text is encode-only. Import (decoding) from plain text is not supported.
- All document structure is lost: headings become plain lines, links lose their URLs, tables lose their structure, and code blocks lose their language annotations.
- Images and other media are omitted entirely from the output.
