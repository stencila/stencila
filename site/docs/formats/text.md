---
title: Plain Text
description: A simple plain text representation of documents
---
# Introduction

Stencila provides support for conversion of documents to plain text. This format is intentionally lossy and only useful when you explicitly want a plain text representation of a document (i.e. lacking structure such as headings and links).

# Usage

Use the `.txt` file extension, or the `--to text` option, when converting to/from plain text e.g.

```sh
stencila convert doc.smd doc.txt
```

# Implementation

Plain text output is implemented in the Rust crate [`codec-text`](https://github.com/stencila/stencila/blob/main/rust/codec-text), using the text trait helpers in [`codec-text-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-text-trait).

# Notes

- Plain text output is lossy by design and drops structural information.
- This format is intended for quick export or copy/paste workflows.
