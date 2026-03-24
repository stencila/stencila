---
title: Pandoc JSON
description: Pandoc's internal document representation in JSON
---

# Introduction

[Pandoc](https://pandoc.org/) is a document converter supporting a wide range of markup formats including Markdown, HTML, LaTeX, DOCX, and ODT. [Pandoc JSON](https://hackage.haskell.org/package/pandoc-types-1.23.1/docs/Text-Pandoc-JSON.html) is a JSON serialization of Pandoc's internal document AST (Abstract Syntax Tree).

Stencila uses Pandoc JSON as an intermediate format when converting to and from formats that Pandoc handles natively, such as [DOCX](../docx), [ODT](../odt), and [PDF](../pdf) (via LaTeX).

# Usage

You are unlikely to need Pandoc JSON directly, but it can be useful for debugging conversion issues. Use the `.pandoc` file extension, or the `--to pandoc` or `--from pandoc` options e.g.

```sh
stencila convert doc.smd doc.pandoc
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and the Pandoc AST powered by the [`pandoc_types`](https://crates.io/crates/pandoc_types) Rust crate. When encoding to Pandoc-based formats (DOCX, ODT), Stencila first converts to the Pandoc AST, serializes it to JSON, and then invokes the `pandoc` binary for final format generation.

# Limitations

- Pandoc must be installed separately; it is not bundled with Stencila.
- The Pandoc AST does not represent all Stencila node types. Conversion is lossy for nodes without Pandoc equivalents (e.g. executable code chunks, styled blocks, parameters).
- Changes to Pandoc's AST between versions may affect compatibility, though Stencila tracks the current stable Pandoc types version.
