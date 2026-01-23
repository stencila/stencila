---
title: Quarto Markdown
description: An extended flavor of Markdown tailored for technical and scientific publishing
---
# Introduction

[Quarto Markdown](https://quarto.org/docs/authoring/markdown-basics.html) is an extended version of Markdown tailored for technical and scientific publishing, enabling reproducible research and literate programming. It builds on the simplicity of standard Markdown by incorporating advanced features like code execution, citations, cross-references, and customizable output formats.

# Usage

To convert to/from Quarto Markdown, use the `.qmd` file extension, or the `--to qmd` or `--from qmd` options e.g.

```sh
stencila convert doc.smd doc.qmd
```

> [!warning]
> Stencila's Quarto Markdown support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Quarto Markdown. See our [CommonMark](../md) documentation for implementation details.

# Notes

- Quarto Markdown is a flavored format; some Stencila nodes may be lossy when round-tripping.
- Quarto-specific metadata and extensions map where possible.
