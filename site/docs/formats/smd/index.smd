---
title: Stencila Markdown
description: An extended flavor of Markdown supporting Stencila's document schema
---

# Introduction

Stencila Markdown (`.smd`) is Stencila's native Markdown flavor. It extends [CommonMark](../md) with block and inline syntax to represent Stencila-specific document elements that standard Markdown cannot express.

Block extensions use `:::` fenced markers (similar to Pandoc divs), while inline extensions use `[[ ... ]]` bracket syntax. Code blocks can be made executable by adding an `exec` flag after the language identifier (e.g. `` ```python exec ``).

Stencila Markdown is the recommended format for authoring documents in Stencila when you want Markdown readability with full access to Stencila Schema features.

# Usage

To convert to/from Stencila Markdown, use the `.smd` file extension, or the `--to smd` or `--from smd` options e.g.

```sh
stencila convert doc.docx doc.smd
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Stencila Markdown. Parsing and encoding are implemented in the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown), with per-node-type Markdown encoding derived via [`codec-markdown-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown-trait) and [`codec-markdown-derive`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown-derive). See the [CommonMark](../md) documentation for details on the underlying Markdown parser.

# Limitations

- Some Stencila node types still have no Markdown representation and are lost when encoding to Stencila Markdown.
- The `:::` and `[[ ... ]]` extension syntax is specific to Stencila and will not render correctly in other Markdown processors.
- Round-tripping through other Markdown tools (e.g. GitHub, VS Code preview) will strip or misrender Stencila extensions.
