---
title: Markdown
description: A lightweight markup language
---
# Introduction

Markdown is a lightweight markup language widely used for formatting plain text documents. It provides a simple and human-readable way to structure text and add basic styling, such as headers, lists, links, and emphasis. Markdown's benefits include ease of use, and compatibility with various web and documentation platforms.

[CommonMark](https://spec.commonmark.org/) is a formal specification that defines a consistent, unambiguous syntax for Markdown, addressing the inconsistencies found in the original Markdown implementation. It serves as a standardization effort to ensure that Markdown content is processed uniformly across different platforms and tools.

In addition to supporting CommonMark, Stencila supports several 'flavors' of Markdown each with extensions to support document elements that are not part of the Commonmark specification:

- [MyST Markdown](../myst)
- [Quarto Markdown](../qmd)
- [Stencila Markdown](../smd)

# Usage

To convert to/from CommonMark, use the `.md` file extension, or the `--to md` or `--from md` options e.g.

```sh
stencila convert doc.smd doc.md
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and CommonMark. Parsing of CommonMark is powered by the [`markdown`](https://crates.io/crates/markdown) Rust crate. Extensions to CommonMark are either supported by the `markdown` crate, or by our own parsing functions, mostly written using the [`winnow`](https://crates.io/crates/winnow) Rust crate.

# Notes

- CommonMark conversion is lossy for nodes that have no markdown representation.
- For richer round-tripping, consider a flavored format such as MyST, Quarto, or Stencila Markdown.
