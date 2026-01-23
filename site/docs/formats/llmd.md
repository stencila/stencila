---
title: LLM Markdown
description: A Markdown flavor optimized for language models
---
# Introduction

LLM Markdown is a Markdown flavor designed for passing content to language models. It favors plain text and reduces syntax that can confuse model inputs.

# Usage

Use the `.llmd` file extension, or the `--to llmd` option, when converting to LLM Markdown e.g.

```sh
stencila convert doc.smd doc.llmd
```

# Implementation

LLM Markdown is implemented by the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown).

# Notes

- LLM Markdown is output-only.
- The format prioritizes readability for models over strict round-tripping.
