# Stencila Markdown

# Introduction

Stencila Markdown is an extended flavor of Markdown designed for executable and structured documents. It keeps Markdown readable while adding block and inline syntax for rich Stencila nodes.

# Usage

Use the `.smd` extension and the `--to smd` or `--from smd` options when converting documents, for example:

```sh
stencila convert doc.docx doc.smd
```

# Implementation

Stencila Markdown is implemented by the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown), together with parsing helpers in [`codec-markdown-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown-trait) and [`codec-markdown-derive`](https://github.com/stencila/stencila/blob/main/rust/codec-markdown-derive).

# Notes

- Block extensions use `:::` markers; inline extensions use `[[ ... ]]` markers.
- Code blocks can be marked executable using a language and an `exec` flag (for example, ```python exec`).
- Some Stencila nodes have no Markdown representation and are lossy when round-tripping.
