---
title: LLM Markdown
description: A Markdown flavor optimized for language model consumption
---

# Introduction

LLM Markdown is a Markdown flavor designed for passing document content to large language models (LLMs). It produces clean, readable plain text that reduces syntax noise, making it easier for models to process document content without being confused by complex markup.

# Usage

Use the `.llmd` file extension, or the `--to llmd` option, when converting to LLM Markdown e.g.

```sh
stencila convert doc.smd doc.llmd
```

> [!info]
> LLM Markdown is encode-only. It is designed for output to language models, not for importing documents.

# Implementation

LLM Markdown encoding is implemented by the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown), using a simplified encoding profile that prioritizes readability for models over round-trip fidelity.

# Limitations

- LLM Markdown is encode-only. There is no corresponding decoder.
- The format is intentionally lossy — structural elements, styling, and executable code metadata are simplified or omitted.
- The output is not standardized and may change between Stencila versions as model input best practices evolve.
