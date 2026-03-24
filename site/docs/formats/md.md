---
title: Markdown
description: A lightweight markup language
---

# Introduction

Markdown is a lightweight markup language widely used for formatting plain text documents. It provides a simple, human-readable way to structure text with headers, lists, links, emphasis, and other basic formatting.

[CommonMark](https://spec.commonmark.org/) is a formal specification that defines a consistent, unambiguous syntax for Markdown, resolving inconsistencies found across different Markdown implementations. Stencila uses CommonMark as the baseline Markdown dialect.

In addition to CommonMark, Stencila supports several Markdown flavors with extensions for document elements not part of the CommonMark specification:

- [MyST Markdown](../myst) — directives and roles for technical documentation
- [Quarto Markdown](../qmd) — code execution and scientific publishing
- [Stencila Markdown](../smd) — Stencila-specific block and inline extensions

For richer round-tripping of Stencila documents, consider using one of these flavored formats instead of plain CommonMark.

# Usage

To convert to/from CommonMark, use the `.md` file extension, or the `--to md` or `--from md` options e.g.

```sh
stencila convert doc.smd doc.md
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and CommonMark. Parsing is powered by the [`markdown`](https://crates.io/crates/markdown) Rust crate. Extensions to CommonMark (e.g. tables, strikethrough, task lists) are either supported by the `markdown` crate or by Stencila's own parsing functions, mostly written using the [`winnow`](https://crates.io/crates/winnow) crate.

# Limitations

- CommonMark conversion is lossy for Stencila node types that have no Markdown representation (e.g. executable code chunks, parameters, styled blocks, admonitions).
- Markdown metadata (YAML front matter) is supported, but only a subset of Stencila Schema properties can be represented.
- Some formatting nuances (e.g. exact whitespace, HTML comments) may not round-trip exactly.
